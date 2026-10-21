//! Zed's trust modal as the Dialogs board draws it (#872).
//!
//! In the Marley layout the modal Zed opens for an untrusted folder asks "Trust repo?" (for a folder named repo) over its
//! path and a line on what trust allows, with Not Now and Trust at the right. Its body (what Restricted Mode
//! prevents, the choice to trust every project in a folder) stays Zed's; a hook in
//! `SecurityModal::render` asks for the two ends.

use std::path::Path;
use std::rc::Rc;

use gpui::{AnyElement, App, FontWeight, Window};
use ui::prelude::*;
use workspace::security_modal::{MarleyTrust, MarleyTrustLook};

use crate::prompts::{Role, render_button};

/// What trust changes, as the board says it.
const LINE: &str = "Trusted, its tasks, language servers and agents can run. Untrusted, you can still read and \
     edit its files.";

/// Sets the hook. [`crate::init`] calls it once.
pub(crate) fn init(cx: &mut App) {
    cx.set_global(MarleyTrustLook(look));
}

/// The modal's header and footer in the Marley layout; none in Zed's, which keeps Zed's modal.
fn look(
    trust: &MarleyTrust<'_>,
    _window: &mut Window,
    cx: &mut App,
) -> Option<(AnyElement, AnyElement)> {
    if !crate::marley_layout(cx) {
        return None;
    }
    let title = match trust.folders {
        [folder] => {
            let name = Path::new(folder).file_name().map_or_else(
                || folder.clone(),
                |name| name.to_string_lossy().into_owned(),
            );
            format!("Trust {name}?")
        }
        folders => format!("Trust these {} projects?", folders.len()),
    };
    let header = v_flex()
        .debug_selector(|| "marley-trust".into())
        .px_3()
        .pt_3()
        .gap_1p5()
        .child(Label::new(title).weight(FontWeight::SEMIBOLD))
        .children(trust.folders.iter().map(|folder| {
            Label::new(folder.clone())
                .size(LabelSize::Small)
                .color(Color::Muted)
        }))
        .child(Label::new(LINE).size(LabelSize::Small).color(Color::Muted));
    let stay = Rc::clone(&trust.stay);
    let accept = Rc::clone(&trust.trust);
    let footer = h_flex()
        .px_3()
        .pb_3()
        .gap_1p5()
        .justify_end()
        .child(
            render_button(0, "Not Now", Role::Quiet, false, cx)
                .on_click(move |_, window, cx| stay(window, cx)),
        )
        .child(
            render_button(1, "Trust", Role::Primary, false, cx)
                .on_click(move |_, window, cx| accept(window, cx)),
        );
    Some((header.into_any_element(), footer.into_any_element()))
}
