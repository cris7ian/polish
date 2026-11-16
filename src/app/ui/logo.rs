use super::*;
use rune::{Easing, LoopSpec};

fn disturb_wave(wave: El, splashes: u64, back: bool) -> El {
    let mut layer = el()
        .key(format!(
            "logo-slosh-{}-{splashes}",
            if back { "back" } else { "front" }
        ))
        .absolute(0.0, 0.0)
        .size(64.0, 32.0)
        .child(wave);
    if splashes > 0 {
        // Disturb only the water layers. The flag and its clipping edge stay still.
        const BACK: &[rune::Beat] = &[
            rune::Beat::new(0.0),
            rune::Beat::new(0.13).scale(1.28),
            rune::Beat::new(0.32).scale(1.02),
            rune::Beat::new(0.53).scale(1.16),
            rune::Beat::new(0.78).scale(1.01),
            rune::Beat::new(1.04).scale(1.07),
            rune::Beat::new(1.4),
        ];
        const FRONT: &[rune::Beat] = &[
            rune::Beat::new(0.0),
            rune::Beat::new(0.10).scale(1.42),
            rune::Beat::new(0.28).scale(1.01),
            rune::Beat::new(0.48).scale(1.23),
            rune::Beat::new(0.72).scale(1.01),
            rune::Beat::new(0.98).scale(1.10),
            rune::Beat::new(1.4),
        ];
        layer = layer.sequence(rune::Sequence::new(if back { BACK } else { FRONT }));
    }
    layer
}
pub(super) fn logo(splashes: u64) -> El {
    el().key("logo")
        .role("button")
        .label("Disturb the water")
        .help("Make a little splash")
        .focusable(true)
        .keeps_text_selection()
        .on_press_tx(|s: &mut State, tx| {
            s.logo_splashes = s.logo_splashes.wrapping_add(1);
            // Replay the water tracks without Rune's default remount crossfade.
            // Fading the white layer exposes the red background across the whole top.
            tx.instant();
        })
        .size(32.0, 32.0)
        .radius(10.0)
        .shadow(2.667, 8.0, color(0x9d3549).alpha(0.133))
        .child(
            el().key("logo-clip")
                .size(32.0, 32.0)
                .radius(10.0)
                .clip_rounded(10.0)
                .gradient(color(0xed5064), color(0xa91e3f))
                .child(disturb_wave(
                    el().key("logo-water-glow")
                        .absolute(0.0, 0.0)
                        .size(64.0, 32.0)
                        .shape(
                            Shape::new().fill_path(
                                &Path::new()
                                    .move_to([0.0, 0.0])
                                    .line_to([64.0, 0.0])
                                    .line_to([64.0, 17.333])
                                    .cubic_to([53.333, 12.0], [42.667, 22.667], [32.0, 17.333])
                                    .cubic_to([21.333, 12.0], [10.667, 22.667], [0.0, 17.333])
                                    .close(),
                                color(0xfff5f5).alpha(0.30),
                            ),
                        )
                        .looping(
                            LoopSpec::slide(7.0, [-0.5, 0.0], [0.0, 0.0]).easing(Easing::LINEAR),
                        ),
                    splashes,
                    true,
                ))
                .child(disturb_wave(
                    el().key("logo-wave")
                        .absolute(0.0, 0.0)
                        .size(64.0, 32.0)
                        .shape(
                            Shape::new().fill_path(
                                &Path::new()
                                    .move_to([0.0, 0.0])
                                    .line_to([64.0, 0.0])
                                    .line_to([64.0, 16.0])
                                    .cubic_to([53.333, 11.6], [43.733, 20.4], [32.0, 16.0])
                                    .cubic_to([21.333, 11.6], [11.733, 20.4], [0.0, 16.0])
                                    .close(),
                                color(0xfbf9f8),
                            ),
                        )
                        .looping(
                            LoopSpec::slide(5.5, [0.0, 0.0], [-0.5, 0.0]).easing(Easing::LINEAR),
                        ),
                    splashes,
                    false,
                ))
                .child(if splashes > 0 {
                    const RIPPLE: &[rune::Beat] = &[
                        rune::Beat::new(0.0).scale(0.3).opacity(0.0),
                        rune::Beat::new(0.08).scale(0.6).opacity(0.65),
                        rune::Beat::new(0.65).scale(2.5).opacity(0.0),
                    ];
                    el().key(format!("logo-ripple-{splashes}"))
                        .absolute(9.0, 14.0)
                        .size(14.0, 5.0)
                        .radius(7.0)
                        .border(0.8, color(0xffffff))
                        .sequence(rune::Sequence::new(RIPPLE))
                        .a11y_hidden()
                } else {
                    el()
                }),
        )
}
