# Marley Licensing & Ownership Strategy

**Question:** We forked the now-open-source Warp terminal (`warpdotdev/warp`) into "Marley."
Warp's client is **AGPLv3**, *except* the UI-framework crates `warpui` / `warpui_core` /
`warpui_extras`, which are **MIT**. If we take the AGPL base and **incrementally rebuild it** —
editing and replacing crates one by one, deleting crates we do not need — until "it isn't really
a fork anymore," do we then **own** the system free of AGPL obligations? What does it actually
take to legally own / relicense it?

> **THIS IS ENGINEERING ANALYSIS, NOT LEGAL ADVICE.** It is written to make a build/architecture
> decision, not to provide a legal opinion. No part of this is a substitute for a real IP attorney.
> **Any commercial relicensing decision MUST be signed off by qualified IP counsel before a single
> dollar of product strategy depends on it.** Where this document says "settled," it means settled
> enough to plan engineering around; where it says "gray," treat it as litigation risk.

---

## 1. Bottom line (blunt)

**No. Incrementally editing, swapping, and deleting crates of the AGPL Warp base — in place, in the
same repository, with the original source on screen — does NOT ever turn it into a work you own free
of the AGPL.** There is **no percentage-replaced threshold**, no "Ship of Theseus" tipping point, and
no in-place-rewrite path that launders copyleft. "It's not really a fork anymore" is **not a legal
test** that exists anywhere in copyright law or in the AGPL.

The reasons, in one breath:

- The AGPL (like the GPL) defines a **"covered work"** as "either the unmodified Program **or a work
  based on the Program**," and defines **"modify"** as "to copy from or adapt all or part of the work
  in a fashion requiring copyright permission."[^agpl] So **editing the original files IS modifying**,
  and the result is a covered work that must stay AGPL as long as **any** protectable Warp expression
  survives in it.
- "Derivative work" is governed by **copyright law, not by the license text** — the FSF/GNU GPL FAQ
  says this explicitly.[^gplfaq] The AGPL therefore reaches **exactly as far as copyright reaches, and
  no further.** That cuts both ways (it's the kernel of truth in the optimistic reading — see §3), but
  for an in-place rewrite it is fatal.
- Your **git history is permanent, dated, authenticated evidence** that you had full access to and
  copied from the AGPL source. In a derivation dispute, **access + copying is the hard element the
  plaintiff usually has to fight for — and a fork concedes it for free.**[^gitlog]

**What actually does let you own it (the only four real routes — see §5):**

1. **Genuine clean-room / independent reimplementation** (write it fresh, from a spec, without
   working from Warp's source). The *only* route that yields code truly free of AGPL.
2. **A commercial / relicense grant from Warp Labs** (the copyright holder is not bound by its own
   AGPL and can dual-license). You cannot self-grant this; there is no public commercial-license
   offer for the Warp **client** today.
3. **Keep it AGPL and comply** (ship the whole derived work under AGPL, including SaaS source
   disclosure under §13). You can build a business, but you do not *own it free of copyleft* and
   competitors can take your source.
4. **Build only on the MIT crates** (`warpui`/`warpui_core`/`warpui_extras`) **plus your own code.**
   Clean and proprietary-compatible — but those crates are just a **UI toolkit**, so everything else
   collapses into route (1).

For a **commercial Ignibyte product**, the realistic combination is **(4) MIT crates + (1) clean-room
reimplementation of the functionality you need**, re-speccing Warp's protocols as functional
interfaces (see §6) — **or** (2) buy a license from Warp. The incremental-rebuild plan is the
**legally dangerous middle that confers no ownership** and produces the **worst possible evidentiary
posture** if challenged.

---

## 2. The derivative-work reality (why "rebuild until it's not a fork" fails)

### 2.1 There is no threshold. It's binary-ish, and it runs against you.

Copyright's actual test for "is this still a derivative?" is **substantial similarity**, decided for
software by the **Abstraction-Filtration-Comparison (AFC) test** from *Computer Associates v.
Altai*.[^altai] AFC abstracts the program into levels, **filters out** everything unprotectable
(ideas, algorithms dictated by efficiency, elements dictated by external factors / interface and
interoperability requirements, *scenes à faire*, public-domain material), and then **compares only
the residual "core of protectable expression."** It is a **qualitative** judgment, case-by-case,
**explicitly not a code-replaced ratio.**[^altai][^afc]

Consequences:

- **"90% replaced" is still 100% encumbered** if the surviving 10% contains protectable Warp
  expression. The work is a derivative as long as **any non-trivial protected expression remains.**
- Conversely, the moment **nothing protectable from the original survives**, it is no longer a covered
  work *at all*. **There is no "partly owned" in-between state.** You only reach the clean end-state by
  effectively *reimplementing* — not by editing in place.
- **Non-literal** structure, sequence and organization (SSO) can survive even a near-total *textual*
  rewrite and still count as copied expression. A low edit-distance or a low plagiarism-tool
  percentage is **not a safe harbor** — AFC is about protected expression, not diff size.

### 2.2 The three buckets of code behave differently

- **DELETED code** simply stops being shipped. You aren't liable for expression you no longer
  distribute — **but deletion does not cleanse what remains.** Removing crates you don't need frees
  *nothing* about the crates you keep.
- **KEPT-AND-EDITED code** stays a derivative for as long as protectable expression traceable to the
  original survives. Editing is **modifying**; it does not detach.[^agpl]
- **NEW code** you add to a still-copyleft codebase is **dragged under the AGPL as part of the
  combined/covered work** for as long as it is distributed as part of that work.[^gplfaq] Mixing new
  code with kept-AGPL crates does **not dilute** the AGPL — it **spreads** it. AGPLv3 §5 requires the
  entire resulting work be licensed as a whole under the AGPL.[^agpl]

### 2.3 You cannot unilaterally relicense the parts you keep

Only the **copyright holder(s)** can relicense. Warp's copyright is owned by **warpdotdev / Denver
Technologies and its contributors** — **forking transfers no copyright.**[^gplfaq][^osguide] You have
a *license* to the AGPL code (conditioned on AGPL compliance); you have **no power to take it
proprietary.**

---

## 3. Where the optimistic reading is actually RIGHT (and where it stops)

Be honest about the kernel of truth, because it defines the legitimate path:

**Copyright protects expression, not ideas, functionality, interfaces, or architecture.** After AFC
filtration, the unprotectable elements — algorithms dictated by efficiency, API/protocol/data-structure
shapes required for interoperability, externally-constrained and public-domain code — **are not owned
by Warp.**[^altai][^afc] A terminal that merely *behaves* like Warp, talks a compatible protocol, and
exposes a compatible command surface owes Warp nothing **if its implementation is independently
authored.** That is the real basis behind "rebuild it until it isn't a fork."

**But the method matters more than the destination.** The optimistic reading is right only for a
**genuinely independent reimplementation** that retains **no** protected expression of the AGPL
crates. It is **wrong** for the proposed method — "incrementally edit/replace crates in the same repo
until it's not a fork" — because:

- Each intermediate commit is *itself* a derivative under AGPL.
- The git history documents **continuous access and copying** commit-by-commit.[^gitlog]
- Non-literal SSO can survive the rewrite and is exactly what a plaintiff points to.
- You demonstrably **started from their code**, which is the **worst** evidentiary posture and the
  **opposite** of the court-tested escape (clean-room). The ambiguity in "which surviving lines are
  protectable expression vs. filtered ideas" is real, but it **favors the copyright holder**, not the
  rewriter, once access + similarity is shown.

An expert as credible as Richard Fontana accepts that **structural independence with no persistent
protected expression can be enough** to escape copyleft.[^willison] But "can be enough" is an
evidentiary standard about *persistence of expression* and *provenance*, **not** "we edited it a lot."
A repo rooted in Warp cannot easily meet it.

---

## 4. AGPL Section 13 specifically — "internal only" is NOT a safe escape for a hosted product

Ordinary GPL copyleft is triggered only by **distribution ("conveying")**. The AGPL adds a **second,
independent trigger** in **§13**. The exact text:[^agpl]

> "Notwithstanding any other provision of this License, **if you modify the Program**, your modified
> version must prominently offer all users interacting with it remotely through a computer network
> (**if your version supports such interaction**) an opportunity to receive the Corresponding Source of
> your version by providing access to the Corresponding Source from a network server at no charge…"

Two prerequisites, both load-bearing:

1. **You must have modified the Program**, AND
2. **the version must support remote network interaction.**

What this means for Marley:

- **The §13 "unmodified" gap is unavailable to you.** It only helps someone who runs the code
  *unmodified*. Marley's entire premise is **heavy modification**. So **any network-service
  deployment of a modified Marley is squarely inside §13** — you must offer Corresponding Source of
  the **entire covered work** to all network users (i.e., your customers). "We never ship a binary /
  it's SaaS" is **not** an escape; §13 exists precisely to close that ASP/SaaS loophole.
- **Shipping Marley as a desktop client** is governed by ordinary **conveying** under §§5–6 — AGPL
  attaches the moment the client is shipped to **anyone outside the company**, and you must offer
  Corresponding Source of the whole covered work. For a forked **desktop terminal**, distribution is
  the operative trigger; §13 layers on top only if you also run a network service.[^agpl][^kemitchell]
- **"Internal use only" is a real escape for plain GPL, but a thin and contested one for AGPL.** It
  holds **only** if there are genuinely no outside network users. The line between safe "internal"
  users (employees) and triggering "outside" users (customers, **contractors**, staffing-agency staff,
  the public) is **fact-specific and contested** — opensource.com flags contractor/staffing access as a
  potential §13 trigger "even if the software never leaves the building."[^opensource] AGPL §13 has
  **very little case law**, so edge interpretations are genuinely gray, and an aggressive holder could
  even argue your configuration/integration counts as "modification."[^kemitchell]

**Net:** For a commercial Ignibyte product, **every realistic shipping shape — desktop client OR hosted
SaaS — triggers full AGPL source-disclosure of the entire derived work.** The only true safe harbor is
"never distributed and no outside network users," which is **incompatible with shipping a product.**

---

## 5. The real ownership paths (with tradeoffs)

| Route | What it gives you | Cost / catch | Realistic for a commercial Ignibyte product? |
|---|---|---|---|
| **(1) Clean-room / independent reimplementation** | Code **genuinely free of AGPL** — you own it, can license it proprietary | Slow, expensive; forfeits the working codebase; demands strict process + contemporaneous evidence; **no defense to patents** | **Yes — the core path**, for the functionality you actually need |
| **(2) Commercial / relicense grant from Warp Labs** | A proprietary license to the **real Warp code** | Entirely at Warp's discretion; **no public commercial-license offer for the client**; Warp uses an agent-first contribution model (no traditional CLA) | Only if Warp chooses to sell; **not on offer today** |
| **(3) Comply with AGPL, stay open** | Legal, zero-permission, ship today | You **do NOT own it free of copyleft**; §13 forces SaaS source disclosure; competitors can take your source | Viable as a business model, **not** as "we own it" |
| **(4) Build only on MIT crates + own code** | Clean, proprietary-compatible base | MIT `warpui*` is **only the UI framework**; everything else you must write yourself → collapses into (1) | **Yes — the clean foundation**, combined with (1) |

### 5.1 Clean-room: what it actually requires (and why the fork can't have it)

Clean-room ("Chinese wall") design is the **only court-tested way** to produce a functionally
compatible work that is **legally independent** rather than derivative. It works by **splitting
personnel**:

- A **"dirty" team** may study the original (or its observable behavior) and writes a **specification
  of WHAT it does** — no protected expression.
- **Legal counsel scrubs** the spec to strip any protected expression.
- A **separate "clean" team that has NEVER seen the original code** implements **solely from the
  vetted spec.**

The legal engine: copyright infringement of software generally requires **access + substantial
similarity** (or direct evidence of copying). **Documented independent creation rebuts the access /
copying prong**, so the remaining similarities are attributed to **functional necessity /
interoperability**, which is uncopyrightable.[^cleanroom] Precedents: **NEC v. Intel** (clean-room
microcode accepted as independent creation), **Phoenix BIOS / Compaq** (the canonical PC-clone
success, deliberately using an engineer with no prior exposure to IBM's BIOS), and **Sega v.
Accolade** / **Sony v. Connectix** (intermediate copying / disassembly to extract *unprotected*
interface elements is fair use **when it's the only way in and the copied material does NOT survive
into the shipped product**).[^cleanroom][^sega][^connectix]

**The evidentiary bar** (you must be able to prove isolation **years later**): per-contributor signed
**taint/NDA declarations** attesting no exposure to Warp source; documented separation of the
spec-writing and implementing teams; **counsel review** of the spec; and **dated** specs, design docs,
and authorship/commit logs showing the clean team built only from the spec. **ReactOS** is the live
example of enforcing exactly this.[^reactos]

**Why incremental forking earns ZERO of this protection:** clean-room's whole point is **no access**
and **no derivation.** Marley begins with **full source access**, a **git-recorded derivation chain**,
and edited code that is a **textbook derivative.** Replacing crates one-by-one does not "launder" the
AGPL; **until 100% of AGPL-derived protected expression is gone, the combined work stays AGPL** — and
clean-room protects **only the crates an isolated team rebuilt from a scrubbed spec.** A single
developer who has read the AGPL source and then "reimplements" is **not** a clean room; you cannot
retrofit the wall after the fact.

**Limits even when done perfectly:** clean-room defeats **copyright/derivative** claims **only**. It is
**no defense to patents** (independent invention doesn't excuse patent infringement) and does nothing
about **trademark** (the Warp name/branding). If Warp's behavior or protocols are patented, even a
flawless rebuild can still infringe.[^cleanroom]

### 5.2 AI-assisted "clean room" is legally untested — do not bank on it

Using an LLM to "regenerate" the copyleft crates into fresh code is cheap, but **no court has ruled**
on whether AI-regenerated code is a derivative of its prompt/training source, whether an AI-assisted
clean room satisfies the independence requirement, or even whether purely AI-generated code is
copyrightable. **Treat "just have Claude rewrite it" as unproven legal risk, not a settled escape** —
and note that feeding the AGPL source into the model is itself **access**, which is exactly what
clean-room is supposed to prevent.

---

## 6. Applied concretely to Marley

**What you CAN do (settled enough to build on):**

- **Re-spec and cleanly reimplement Warp's multi-agent protobuf protocol.** Under **17 U.S.C.
  §102(b)**, ideas, procedures, processes, systems, and **methods of operation** are categorically
  uncopyrightable.[^102b] Wire protocols, message/field names, field types, and the wire format are
  **functional method-of-operation**, and where there is essentially **one way to express the interface
  for interoperability**, the **merger doctrine** strips protection.[^merger] *Google v. Oracle*
  reinforces this direction: the Court held that even verbatim copying of ~11,500 lines (~0.4%) of Java
  API *declaring code* to reimplement an interface was **fair use**, calling declaring code
  "inextricably bound together" with uncopyrightable methods of operation — though note it **expressly
  declined to rule APIs uncopyrightable**, deciding only on fair use, so a sliver of doctrinal
  uncertainty remains.[^oracle] *Lotus v. Borland* (command hierarchy = uncopyrightable method of
  operation) and *Sega/Connectix* (functional interoperability requirements are unprotected) point the
  same way.[^lotus][^sega][^connectix] **Best practice:** reauthor your **own `.proto`** from the
  observed wire behavior / spec — **do not copy Warp's `.proto` file verbatim**, because doc comments,
  descriptive prose, examples, and non-functional ordering can carry **thin copyright**, and that file
  most likely sits under the **AGPL** client, not the MIT crates.
- **Reuse the MIT `warpui` / `warpui_core` / `warpui_extras` crates** freely and keep/relicense your
  combination as proprietary, subject only to MIT's **attribution/notice-retention** requirement.[^warp]
  MIT is **orthogonal** to the AGPL question: it neither helps nor hurts whether the AGPL-derived parts
  are cleansed — it just means the UI framework carries no copyleft.

**What you CANNOT do:**

- **Copy Warp's Rust IMPLEMENTATION source** for anything outside the MIT crates. That is copyrightable
  expression, it is AGPL-covered, and paraphrasing/translating it (same structure, sequence,
  organization beyond what the interface dictates) is **still a derivative** — interface freedom does
  **not** launder copied implementation.
- **Escape the AGPL by editing the AGPL crates in place.** Re-speccing the protocol does **not** pull
  in AGPL; **editing the AGPL base DOES.** The copyright-clean path is **independent reimplementation
  behind a re-specced interface**, not in-place editing of forked crates.[^gplfaq]

**The git fork is permanent provenance.** The public, dated commit history rooted in `warpdotdev/warp`
is **direct evidence of access and the chronology of copying** — it concedes the element a plaintiff
normally has to prove, and you should assume it will be in front of a fact-finder (discovery of
version-control history is discretionary and has gone **both** ways).[^gitlog] **Provenance hygiene
matters as much as the diff:** prior maintainer knowledge of the codebase, models trained on Warp's
source, and stray references to the originals all erode an "independent creation" defense even if the
final bytes differ.

### 6.1 Recommended posture for a commercial Marley

1. **Sever the fork.** Treat the current Marley fork as a **reference/"dirty"-team artifact only** —
   not as the product trunk. Do **not** ship from it.
2. **Stand up a fresh repository** for the product with **no git ancestry** from `warpdotdev/warp`.
3. **Foundation = MIT `warpui*` crates + your own code** (route 4).
4. **Re-spec the protobuf protocol** into your **own** `.proto`, authored from spec/observed behavior,
   reviewed by counsel (route via §102(b)/merger).
5. **Clean-room reimplement** the non-UI functionality you need, with the personnel wall and
   contemporaneous evidence described in §5.1 (route 1) — **or** open a conversation with Warp Labs
   about a **commercial license** (route 2).
6. If none of that is affordable, **ship under the AGPL and comply** (route 3) — a legitimate business,
   but you do **not** own it free of copyleft, and §13 means your SaaS must disclose source to users.
7. **Get IP counsel to bless the process before commercializing.** Especially: the clean-room wall, the
   §13 SaaS analysis, the protobuf re-spec, and **patent/trademark** clearance (which clean-room does
   **not** solve).

---

## 7. Confidence & gray areas

**High confidence (settled enough to build engineering on):**

- In-place incremental editing does **not** escape the AGPL; there is **no percentage threshold**.
  (GPL/AGPL text + GNU FAQ; AFC test.)[^agpl][^gplfaq][^altai]
- The AGPL reaches exactly as far as **copyright's derivative-work boundary**, no further.[^gplfaq]
- AGPL §13 makes **modified SaaS** trigger source disclosure to network users; "internal only" is the
  only true safe harbor and is incompatible with shipping.[^agpl]
- **Interfaces / protocols / wire formats are functional** and largely outside copyright; you can
  **re-spec and reimplement** Warp's protocol cleanly.[^102b][^lotus][^oracle]
- **Clean-room** is the recognized escape; **incremental forking is its antithesis** and earns none of
  its protection.[^cleanroom][^reactos]
- The **MIT `warpui*` carve-out** is genuinely reusable in a proprietary product.[^warp]
- Only **Warp Labs** can relicense Warp's code; a forker cannot self-grant.[^gplfaq]

**Gray areas (real litigation risk — get counsel):**

- **How much non-literal SSO** survives a given rewrite, and whether residual fragments are
  "protectable expression" vs. filtered ideas — **fact-intensive, litigated per-file.**
- The **§13 "modification" gloss** is FSF/practitioner consensus, **not** black-letter litigated law;
  AGPL §13 has thin case law and edge readings are open.[^kemitchell]
- The **employee-vs-contractor** line for "internal" AGPL use.[^opensource]
- *Google v. Oracle* is a **fair-use** ruling, **not** a holding that APIs are uncopyrightable —
  residual doctrinal uncertainty on schema expression remains.[^oracle]
- **AI-assisted clean room** is **legally untested.**
- "Structural independence with no persistent expression suffices" is credible (Fontana) but must be
  **demonstrable**, and a Warp-rooted history undercuts it.[^willison]
- **Patents and trademark** are **separate regimes** clean-room does **not** address. Possible
  **DMCA §1201 / EULA** anti-reverse-engineering wrinkles if any protection measure or contract term is
  implicated.

> **Reminder:** This is engineering analysis to guide an architecture decision. **It is not legal
> advice, and a qualified IP attorney must sign off before any commercial relicensing or
> productization decision.**

---

## Sources

[^agpl]: GNU Affero General Public License v3.0, full text (Section 0 definitions of "modify",
"modified version", "covered work", "conveying"; Section 5; **Section 13** "Remote Network
Interaction"). https://www.gnu.org/licenses/agpl-3.0.html
[^gplfaq]: GNU GPL Frequently Asked Questions — "derivative work" is defined by copyright law not the
license; new code distributed with a covered work is pulled under the license; only copyright holders
can relicense. https://www.gnu.org/licenses/gpl-faq.html
[^altai]: *Computer Associates International, Inc. v. Altai, Inc.* (2d Cir. 1992) — Abstraction-
Filtration-Comparison test for software substantial similarity.
https://www.bitlaw.com/source/cases/copyright/altai.html ;
https://en.wikipedia.org/wiki/Computer_Associates_International,_Inc._v._Altai,_Inc.
[^afc]: Abstraction-Filtration-Comparison test (overview).
https://en.wikipedia.org/wiki/Abstraction-Filtration-Comparison_test
[^gitlog]: Git history as legal evidence of access/derivation; discovery of version-control history is
discretionary and has gone both ways. https://bojanjosifoski.com/your-git-log-is-a-legal-document/ ;
https://ipde.com/blog/2024/09/25/court-denies-request-for-discovery-of-git-version-control-history-for-source-code/
[^cleanroom]: Clean-room (Chinese-wall) design — two-team process, access+substantial-similarity
mechanism, NEC v. Intel, Phoenix BIOS; no defense to patents.
https://en.wikipedia.org/wiki/Clean_room_design ;
https://en.wikipedia.org/wiki/Phoenix_Technologies
[^reactos]: ReactOS clean-room enforcement (taint declarations, source-review freeze).
https://en.wikipedia.org/wiki/ReactOS ;
https://reactos.org/project-news/reset-reboot-restart-legal-issues-and-long-road-03/
[^sega]: *Sega Enterprises Ltd. v. Accolade, Inc.* (9th Cir. 1992) — intermediate copying to extract
unprotected interface elements is fair use. https://en.wikipedia.org/wiki/Sega_v._Accolade ;
https://www.copyright.gov/fair-use/summaries/segaenters-accolade-9thcir1992.pdf
[^connectix]: *Sony Computer Entertainment, Inc. v. Connectix Corp.* (9th Cir. 2000).
https://en.wikipedia.org/wiki/Sony_Computer_Entertainment,_Inc._v._Connectix_Corp. ;
https://euro.ecom.cmu.edu/program/law/08-732/Copyright/SonyVConnectix.pdf
[^102b]: 17 U.S.C. §102(b) — ideas, procedures, processes, systems, methods of operation excluded from
copyright. https://www.law.cornell.edu/uscode/text/17/102
[^merger]: Merger doctrine — no protection where there is essentially one way to express an
idea/interface. https://websites.umass.edu/copyright/copyright-basics/copyrightability/
[^oracle]: *Google LLC v. Oracle America, Inc.*, 593 U.S. ___ (2021) — copying API declaring code to
reimplement an interface was fair use; Court declined to decide API copyrightability.
https://www.supremecourt.gov/opinions/20pdf/18-956_d18f.pdf ;
https://www.eff.org/deeplinks/2021/04/victory-fair-use-supreme-court-reverses-federal-circuit-oracle-v-google
[^lotus]: *Lotus Dev. Corp. v. Borland Int'l, Inc.* (1st Cir. 1995) — menu/command hierarchy is an
uncopyrightable method of operation. https://www.bitlaw.com/source/cases/copyright/Lotus.html
[^opensource]: opensource.com — "Providing corresponding source under AGPLv3"; §13 prerequisites and
the contractor/internal-use gray line.
https://opensource.com/article/17/1/providing-corresponding-source-agplv3-license
[^kemitchell]: Kyle E. Mitchell, "Reading the AGPL" — §13 modification + network-interaction
prerequisites; thin case law. https://writing.kemitchell.com/2021/01/24/Reading-AGPL
[^warp]: Warp open-source release — client under AGPLv3, UI framework crates
(`warpui`/`warpui_core`/`warpui_extras`) under MIT; server/AI cloud remain proprietary.
https://www.warp.dev/blog/warp-is-now-open-source ;
https://github.com/warpdotdev/warp ;
https://www.helpnetsecurity.com/2026/04/30/warp-open-source-client/
[^osguide]: GitHub Open Source Guide — forking transfers no copyright; copyright owned by authors.
https://opensource.guide/legal/
[^willison]: Simon Willison, notes on chardet/structural independence (Fontana view that structural
independence with no persistent expression can suffice). https://simonwillison.net/2026/Mar/5/chardet/
[^sflc]: Software Freedom Law Center, "A Legal Issues Primer / Guide to GPL Compliance" (2d ed.) —
covered-work scope and derivative boundary.
https://softwarefreedom.org/resources/2014/SFLC-Guide_to_GPL_Compliance_2d_ed.html
