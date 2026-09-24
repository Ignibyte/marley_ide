// seat-pointer WIDTH HEIGHT: gives a compositor's seat a pointer that an e2e scenario drives.
//
// A headless sway starts with no input devices, so its seat has no pointer and no client ever
// gets a wl_pointer. This client creates one through the wlr-virtual-pointer protocol and holds
// it, reading one command a line from stdin and answering `ok` (or `error`) after each, until
// stdin closes:
//
//   move X Y        absolute motion to X, Y in an output of WIDTH x HEIGHT pixels
//   down [BUTTON]   press left (the default), middle or right
//   up [BUTTON]     release it
//   scroll STEPS    wheel detents, positive down
//
// script/e2e.sh builds it on first use with wayland-scanner and cc (CONSTITUTION §7, #487).
#include <linux/input-event-codes.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>
#include <wayland-client.h>

#include "wlr-virtual-pointer-unstable-v1-client-protocol.h"

static struct wl_seat *seat;
static struct zwlr_virtual_pointer_manager_v1 *manager;

static void add_global(void *data, struct wl_registry *registry, uint32_t name,
                       const char *interface, uint32_t version) {
    if (strcmp(interface, wl_seat_interface.name) == 0 && !seat) {
        seat = wl_registry_bind(registry, name, &wl_seat_interface, 1);
    } else if (strcmp(interface, zwlr_virtual_pointer_manager_v1_interface.name) == 0) {
        // Version 2 brings the wheel's axis source and discrete steps.
        manager = wl_registry_bind(registry, name, &zwlr_virtual_pointer_manager_v1_interface,
                                   version < 2 ? version : 2);
    }
}

static void remove_global(void *data, struct wl_registry *registry, uint32_t name) {}

static const struct wl_registry_listener registry_listener = {add_global, remove_global};

static uint32_t now_ms(void) {
    struct timespec now;
    clock_gettime(CLOCK_MONOTONIC, &now);
    return (uint32_t)(now.tv_sec * 1000 + now.tv_nsec / 1000000);
}

static int button_code(const char *name, uint32_t *code) {
    if (strcmp(name, "left") == 0) {
        *code = BTN_LEFT;
    } else if (strcmp(name, "middle") == 0) {
        *code = BTN_MIDDLE;
    } else if (strcmp(name, "right") == 0) {
        *code = BTN_RIGHT;
    } else {
        return -1;
    }
    return 0;
}

static void answer(const char *reply) {
    printf("%s\n", reply);
    fflush(stdout);
}

int main(int argc, char **argv) {
    if (argc != 3) {
        fprintf(stderr, "usage: %s WIDTH HEIGHT\n", argv[0]);
        return 2;
    }
    int width = atoi(argv[1]);
    int height = atoi(argv[2]);
    if (width <= 0 || height <= 0) {
        fprintf(stderr, "seat-pointer: the output's size must be positive\n");
        return 2;
    }
    struct wl_display *display = wl_display_connect(NULL);
    if (!display) {
        fprintf(stderr, "seat-pointer: no Wayland display\n");
        return 1;
    }
    struct wl_registry *registry = wl_display_get_registry(display);
    wl_registry_add_listener(registry, &registry_listener, NULL);
    if (wl_display_roundtrip(display) < 0 || !seat || !manager) {
        fprintf(stderr, "seat-pointer: the compositor offers no seat or no virtual pointer\n");
        return 1;
    }
    struct zwlr_virtual_pointer_v1 *pointer =
        zwlr_virtual_pointer_manager_v1_create_virtual_pointer(manager, seat);
    if (wl_display_roundtrip(display) < 0) {
        fprintf(stderr, "seat-pointer: the compositor refused the pointer\n");
        return 1;
    }
    answer("ready");

    char line[256];
    while (fgets(line, sizeof line, stdin)) {
        char verb[16] = "";
        char button[16] = "left";
        double x, y, steps;
        uint32_t code;
        if (sscanf(line, "move %lf %lf", &x, &y) == 2 && x >= 0 && y >= 0) {
            zwlr_virtual_pointer_v1_motion_absolute(pointer, now_ms(), (uint32_t)x, (uint32_t)y,
                                                    (uint32_t)width, (uint32_t)height);
        } else if (sscanf(line, "scroll %lf", &steps) == 1) {
            // A wheel detent is 15 units of axis value, as libinput reports one.
            zwlr_virtual_pointer_v1_axis_source(pointer, WL_POINTER_AXIS_SOURCE_WHEEL);
            zwlr_virtual_pointer_v1_axis_discrete(pointer, now_ms(),
                                                  WL_POINTER_AXIS_VERTICAL_SCROLL,
                                                  wl_fixed_from_double(steps * 15),
                                                  (int32_t)steps);
        } else if (sscanf(line, "%15s %15s", verb, button) >= 1 &&
                   (strcmp(verb, "down") == 0 || strcmp(verb, "up") == 0) &&
                   button_code(button, &code) == 0) {
            zwlr_virtual_pointer_v1_button(pointer, now_ms(), code,
                                           strcmp(verb, "down") == 0
                                               ? WL_POINTER_BUTTON_STATE_PRESSED
                                               : WL_POINTER_BUTTON_STATE_RELEASED);
        } else {
            fprintf(stderr, "seat-pointer: unknown command: %s", line);
            answer("error");
            continue;
        }
        zwlr_virtual_pointer_v1_frame(pointer);
        if (wl_display_roundtrip(display) < 0) {
            fprintf(stderr, "seat-pointer: the compositor went away\n");
            return 1;
        }
        answer("ok");
    }
    zwlr_virtual_pointer_v1_destroy(pointer);
    wl_display_roundtrip(display);
    wl_display_disconnect(display);
    return 0;
}
