use crate::{
    model::{Doc, Shelf},
    store::{SaveStatus, store},
};
use nsh_base::input;
use nsh_core::prelude::*;
const F: [(&str, &str); 4] = [
    ("宋体", "Noto Serif SC"),
    ("楷体", "KaiTi"),
    ("仿宋", "FangSong"),
    ("黑体", "Microsoft YaHei"),
];
const INK: u32 = 0x24282c;
const SLATE: u32 = 0x43586e;
const BLUE: u32 = 0x294c75;
const PAPER: u32 = 0xf9f6f2;
fn shelf() -> Signal<Shelf> {
    global::<Shelf>()
}
fn current<T>(f: impl FnOnce(&Doc) -> T) -> T {
    shelf().with(|s| f(&s.docs[s.active]))
}
fn face() -> SharedString {
    current(|d| F[d.font].1.into())
}
fn index() -> String {
    shelf().with(|s| format!("{:02}", s.active + 1))
}
fn save(change: impl FnOnce(&mut Shelf)) {
    shelf().update(change);
    store().submit(shelf().with_untracked(Clone::clone));
    global::<SaveStatus>().set(SaveStatus::Saving);
}
fn edit(f: impl FnOnce(&mut Doc)) {
    save(|s| f(&mut s.docs[s.active]))
}
fn create() {
    save(|s| {
        s.docs.push(Doc {
            title: "新文稿".into(),
            body: "".into(),
            date: chrono::Local::now().format("%Y-%m-%d").to_string(),
            font: s.last,
        });
        s.active = s.docs.len() - 1
    })
}
fn sw() -> f32 {
    (window().width.0 * 0.287).clamp(228., 350.)
}
fn tx(t: &'static str, size: f32, f: &'static str, c: u32) -> Text {
    text(t)
        .text_size(px(size))
        .font_family(f)
        .text_color(rgb(c))
}
fn click(v: View, label: &'static str, fun: impl Fn() + 'static) -> View {
    v.role(AccessibilityRole::Button).label(label).on_press(fun)
}
fn pill() -> View {
    view()
        .rounded(px(18.))
        .border_1()
        .border_color(rgb(0xd4dce1))
        .bg(rgb(0xe9eef1))
        .px(px(15.))
        .h(px(32.))
        .items_center()
        .justify_center()
}
#[derive(Clone, Copy, Default, PartialEq)]
struct Optical {
    mode: i32,
    phase: f32,
    tilt: f32,
    paused: bool,
}
fn opt() -> Signal<Optical> {
    global::<Optical>()
}
fn layer(src: &str) -> impl IntoElement {
    use nsh_core::shader::{Shader, UniformValue};
    shader()
        .source(Shader::compile(src).expect("valid WGSL"))
        .absolute()
        .w_full()
        .h_full()
        .uniform("mode", || UniformValue::Number(opt().get().mode as f32))
        .uniform("phase", || UniformValue::Number(opt().get().phase))
        .uniform("tilt", || UniformValue::Number(opt().get().tilt))
        .uniform("sourceKind", 0_f32)
}
fn sidebar(menu: Signal<bool>) -> View {
    let brand = view()
        .flex_row()
        .items_center()
        .gap(px(15.))
        .pl(px(9.))
        .child(
            view()
                .size(px(39.))
                .rounded(px(21.))
                .items_center()
                .justify_center()
                .bg(rgb(0x435f87))
                .child(tx("≈", 30., "Georgia", 0xffffff)),
        )
        .child(
            view()
                .child(tx("雾笺", 27., F[0].1, 0x203341))
                .child(tx("MISTPAPER", 10., "Georgia", SLATE).letter_spacing(px(3.))),
        );
    let header = view()
        .flex_row()
        .items_center()
        .mt(px(37.))
        .mb(px(16.))
        .px(px(10.))
        .child(tx("文稿", 15., F[0].1, SLATE))
        .child(
            tx("/  LIBRARY", 10., "Georgia", SLATE)
                .flex_1()
                .ml(px(12.))
                .letter_spacing(px(2.)),
        )
        .child(
            click(
                view().w(px(28.)).h(px(30.)).items_center().justify_center(),
                "新建文稿",
                move || {
                    menu.set(false);
                    create()
                },
            )
            .child(tx("＋", 24., "system-ui", SLATE)),
        );
    let rows = each(
        || shelf().with(|s| (0..s.docs.len()).collect::<Vec<_>>()),
        |i| *i,
        move |idx, _| {
            let i = idx.get_untracked();
            let active = move || shelf().with(|s| s.active == i);
            view()
                .role(AccessibilityRole::Button)
                .label(move || shelf().with(|s| s.docs[i].title.clone()))
                .on_press(move || {
                    menu.set(false);
                    save(|s| s.active = i)
                })
                .h(px(42.))
                .mb(px(3.))
                .px(px(10.))
                .border_1()
                .rounded(px(9.))
                .border_color(move || {
                    if active() {
                        rgba(0xeff8ffb0)
                    } else {
                        rgba(0xeef6fb77)
                    }
                })
                .bg(move || if active() { rgba(0xb6d3eb9c) } else { rgba(0) })
                .flex_row()
                .items_center()
                .gap(px(10.))
                .child(
                    text(format!("{:02}", i + 1))
                        .font_family("Georgia")
                        .text_size(px(13.))
                        .text_color(rgb(SLATE)),
                )
                .child(view().h(px(14.)).w(px(1.)).bg(rgb(0xa9bccb)))
                .child(
                    text(move || shelf().with(|s| s.docs[i].title.clone()))
                        .flex_1()
                        .font_family(F[0].1)
                        .text_size(px(16.))
                        .text_color(rgb(INK)),
                )
                .child(
                    text(move || {
                        shelf().with(|s| s.docs[i].date.get(5..).unwrap_or("").replace('-', "."))
                    })
                    .font_family("Georgia")
                    .text_size(px(12.))
                    .text_color(rgb(SLATE)),
                )
        },
    );
    let foot = view()
        .mx(px(10.))
        .mb(px(22.))
        .pt(px(12.))
        .border_t(px(1.))
        .border_color(rgb(0xedf5fa))
        .flex_row()
        .items_center()
        .gap(px(9.))
        .child(view().size(px(7.)).rounded(px(4.)).bg(|| {
            if global::<SaveStatus>().get() == SaveStatus::Failed {
                rgb(0xae8652)
            } else {
                rgb(0x60a56c)
            }
        }))
        .child(
            text(|| {
                if global::<SaveStatus>().get() == SaveStatus::Failed {
                    "保存失败"
                } else {
                    match global::<SaveStatus>().get() {
                        SaveStatus::Saving => "保存中",
                        _ => "本地保存",
                    }
                }
            })
            .font_family(F[0].1)
            .text_size(px(12.))
            .text_color(rgb(SLATE)),
        );
    view()
        .w(|| px(sw()))
        .h_full()
        .overflow_hidden()
        .bg(rgb(0xd1deea))
        .on_mouse_move(|e: &MouseEvent| {
            let t = (((2. * e.position.x.0 / sw() - 1.).clamp(-1., 1.)) * 8.).round() / 8.;
            let drag = e.pressed_button.is_some();
            let phase = (e.position.x.0 / 2.).round();
            if opt().with_untracked(|o| o.tilt != t || (drag && o.phase != phase)) {
                opt().update(|o| {
                    o.tilt = t;
                    if drag {
                        o.phase = phase
                    }
                })
            }
        })
        .on_mouse_leave(|_: &MouseEvent| opt().update(|o| o.tilt = 0.))
        .child(layer(include_str!("../assets/shaders/glass.wgsl")))
        .child(
            view()
                .px(px(27.))
                .pt(px(45.))
                .pb(px(10.))
                .flex_1()
                .child(brand)
                .child(header)
                .child(scroll().flex_1().child(rows))
                .child(foot),
        )
}
fn menu_layer(menu: Signal<bool>) -> View {
    let mut v = view()
        .absolute()
        .top(px(53.))
        .right(px(106.))
        .w(px(135.))
        .p(px(6.))
        .rounded(px(12.))
        .border_1()
        .border_color(rgb(0xd1dbe3))
        .bg(rgb(0xf1f5f8));
    for (i, (name, f)) in F.iter().copied().enumerate() {
        v = v.child(
            click(
                view().px(px(9.)).py(px(7.)).rounded(px(6.)).bg(move || {
                    if current(|d| d.font) == i {
                        rgb(0xdce7f0)
                    } else {
                        rgba(0)
                    }
                }),
                name,
                move || {
                    save(|s| {
                        s.last = i;
                        s.docs[s.active].font = i
                    });
                    menu.set(false)
                },
            )
            .child(tx(name, 15., f, INK)),
        )
    }
    v
}
#[component]
pub fn App() -> impl IntoElement {
    shelf().set(store().initial());
    let focus = signal(false);
    let menu = signal(false);
    let tick = stored(std::time::Instant::now());
    set_interval(std::time::Duration::from_millis(33), move || {
        let now = std::time::Instant::now();
        let dt = now.duration_since(tick.replace(now)).as_secs_f32().min(0.1);
        if !focus.get_untracked() && !opt().with_untracked(|o| o.paused) {
            opt().update(|o| {
                o.phase = (o.phase + dt * 3.).rem_euclid(window().height.0.max(1.) * 6.)
            });
        }
        let status = store().status();
        if status != global::<SaveStatus>().get_untracked() {
            global::<SaveStatus>().set(status);
        }
    });
    let toolbar = view()
        .border_b(px(1.))
        .border_color(rgb(0xe8e5df))
        .h(px(62.))
        .px(px(25.))
        .items_center()
        .flex_row()
        .gap(px(9.))
        .child(
            text(|| format!("随笔集　/　{}", index()))
                .flex_1()
                .font_family(F[0].1)
                .text_size(px(14.))
                .text_color(rgb(INK)),
        )
        .child(
            click(pill(), "切换玻璃", || {
                opt().update(|o| o.mode = (o.mode + 1) % 3)
            })
            .child(
                text(|| ["透镜", "无折射", "原景"][opt().get().mode as usize])
                    .text_size(px(13.))
                    .text_color(rgb(SLATE)),
            ),
        )
        .child(
            click(pill(), "切换山景动画", || {
                opt().update(|o| o.paused = !o.paused)
            })
            .child(
                text(|| {
                    if opt().get().paused {
                        "流动"
                    } else {
                        "暂停"
                    }
                })
                .text_size(px(13.))
                .text_color(rgb(SLATE)),
            ),
        )
        .child(
            click(pill(), "选择字体", move || menu.update(|v| *v = !*v)).child(
                text(|| format!("{}　▾", current(|d| F[d.font].0)))
                    .font_family(F[0].1)
                    .text_size(px(13.))
                    .text_color(rgb(SLATE)),
            ),
        )
        .child(
            click(pill(), "专注模式", move || {
                focus.update(|v| *v = !*v);
                menu.set(false)
            })
            .child(
                text(move || {
                    if focus.get() {
                        "⛶　退出"
                    } else {
                        "⛶　专注"
                    }
                })
                .font_family(F[0].1)
                .text_size(px(13.))
                .text_color(rgb(SLATE)),
            ),
        );
    let column = view()
        .w(move || {
            px((window().width.0 - if focus.get() { 12. } else { sw() } - 60.).clamp(180., 500.))
        })
        .flex_1()
        .child(
            text(|| format!("FIELD NOTES　/　{}", index()))
                .font_family("Georgia")
                .text_size(px(12.))
                .letter_spacing(px(2.8))
                .text_color(rgb(BLUE)),
        )
        .child(
            input()
                .label("文稿标题")
                .value(|| current(|d| d.title.clone()))
                .on_change(|v: &str| edit(|d| d.title = v.into()))
                .mt(px(16.))
                .p(px(0.))
                .w_full()
                .h(px(51.))
                .font_family(face)
                .text_size(px(37.))
                .text_color(rgb(INK))
                .bg(rgb(PAPER)),
        )
        .child(
            text(|| {
                current(|d| {
                    let x: Vec<_> = d.date.split('-').collect();
                    if x.len() == 3 {
                        format!("{}年{}月{}日", x[0], x[1], x[2])
                    } else {
                        d.date.clone()
                    }
                })
            })
            .font_family(F[0].1)
            .text_size(px(14.))
            .text_color(rgb(0x65686a))
            .mt(px(8.)),
        )
        .child(
            view()
                .w(px(38.))
                .h(px(2.))
                .bg(rgb(BLUE))
                .mt(px(20.))
                .mb(px(23.)),
        )
        .child(
            input()
                .multiline(true)
                .scrollbar(false)
                .label("正文")
                .placeholder("在这里开始写作……")
                .value(|| current(|d| d.body.to_string()))
                .on_change(|v: &str| edit(|d| d.body = v.into()))
                .w_full()
                .flex_1()
                .min_h(px(60.))
                .p(px(0.))
                .font_family(face)
                .text_size(px(20.))
                .line_height(px(30.))
                .letter_spacing(px(0.))
                .text_color(rgb(INK))
                .bg(rgb(PAPER)),
        );
    view()
        .flex_1()
        .flex_row()
        .bg(rgb(0xc3d3de))
        .child(when(move || !focus.get(), move || sidebar(menu)))
        .child(
            view()
                .flex_1()
                .my(px(12.))
                .mr(px(12.))
                .ml(move || px(if focus.get() { 12. } else { 0. }))
                .rounded(px(19.))
                .overflow_hidden()
                .bg(rgb(PAPER))
                .border_1()
                .border_color(rgb(0xeff3f4))
                .child(toolbar)
                .child(
                    view()
                        .flex_1()
                        .items_center()
                        .pt(|| px((window().height.0 * 0.12 - 48.).clamp(10., 48.)))
                        .child(column),
                )
                .child(
                    text(|| shelf().with(|s| format!("{:02} / {:02}", s.active + 1, s.docs.len())))
                        .self_end()
                        .mr(px(25.))
                        .my(px(14.))
                        .font_family("Georgia")
                        .text_size(px(12.))
                        .letter_spacing(px(1.))
                        .text_color(rgb(0x697078)),
                )
                .child(when(menu, move || menu_layer(menu))),
        )
}
