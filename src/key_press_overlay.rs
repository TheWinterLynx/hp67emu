use eframe::egui::{Align2, Color32, FontFamily, FontId, Painter, Pos2, Rect, Stroke, Ui, Vec2};

const DESIGN_W: f32 = 330.0;
const DESIGN_H: f32 = 620.0;
const PANEL: Color32 = Color32::from_rgb(45, 46, 43);
const WHITE: Color32 = Color32::from_rgb(238, 239, 232);
const DARK: Color32 = Color32::from_rgb(27, 29, 28);

#[derive(Clone, Copy)]
enum KeyStyle { Olive, Orange, Blue, White, Black }

#[derive(Clone, Copy)]
enum SubAlign { Center, Right }

#[derive(Clone, Copy)]
struct KeySpec {
    id: &'static str,
    cx: f32,
    y: f32,
    w: f32,
    h: f32,
    top_h: f32,
    style: KeyStyle,
    main: &'static str,
    sub: Option<&'static str>,
    sub_align: SubAlign,
}

const fn k(id: &'static str, cx: f32, y: f32, w: f32, h: f32, top_h: f32,
    style: KeyStyle, main: &'static str, sub: Option<&'static str>, sub_align: SubAlign) -> KeySpec {
    KeySpec { id, cx, y, w, h, top_h, style, main, sub, sub_align }
}

const KEYS: &[KeySpec] = &[
    k("a",65.0,169.5,35.0,29.5,20.0,KeyStyle::Olive,"A",None,SubAlign::Center),
    k("b",116.0,169.5,35.0,29.5,20.0,KeyStyle::Olive,"B",None,SubAlign::Center),
    k("c",167.0,169.5,35.0,29.5,20.0,KeyStyle::Olive,"C",None,SubAlign::Center),
    k("d",218.0,169.5,35.0,29.5,20.0,KeyStyle::Olive,"D",None,SubAlign::Center),
    k("e",269.0,169.5,35.0,29.5,20.0,KeyStyle::Olive,"E",None,SubAlign::Center),
    k("sigma",65.0,223.2,35.0,31.0,19.1,KeyStyle::Olive,"Σ+",Some("Σ−"),SubAlign::Center),
    k("gto",116.0,223.2,35.0,31.0,19.1,KeyStyle::Olive,"GTO",Some("RTN"),SubAlign::Center),
    k("dsp",167.0,223.2,35.0,31.0,19.1,KeyStyle::Olive,"DSP",Some("ENG"),SubAlign::Center),
    k("indirect",218.0,223.2,35.0,31.0,19.1,KeyStyle::Olive,"(i)",Some("x↔I"),SubAlign::Center),
    k("sst",269.0,223.2,35.0,31.0,19.1,KeyStyle::Olive,"SST",Some("BST"),SubAlign::Center),
    k("f",65.0,278.0,35.0,29.5,20.0,KeyStyle::Orange,"f",None,SubAlign::Center),
    k("g",116.0,278.0,35.0,29.5,20.0,KeyStyle::Blue,"g",None,SubAlign::Center),
    k("sto",167.0,278.0,35.0,31.0,19.1,KeyStyle::Olive,"STO",Some("ST I"),SubAlign::Center),
    k("rcl",218.0,278.0,35.0,31.0,19.1,KeyStyle::Olive,"RCL",Some("RC I"),SubAlign::Center),
    k("h",269.0,278.0,35.0,29.5,20.0,KeyStyle::Black,"h",None,SubAlign::Center),
    k("enter",90.0,332.2,84.0,31.0,19.5,KeyStyle::Olive,"ENTER",Some("DEG"),SubAlign::Right),
    k("chs",167.0,332.2,35.0,31.0,19.1,KeyStyle::Olive,"CHS",Some("RAD"),SubAlign::Center),
    k("eex",218.0,332.2,35.0,31.0,19.1,KeyStyle::Olive,"EEX",Some("GRD"),SubAlign::Center),
    k("clx",269.0,332.2,35.0,31.0,19.1,KeyStyle::Olive,"CLx",Some("DEL"),SubAlign::Center),
    k("minus",60.5,384.6,27.0,30.5,18.6,KeyStyle::Olive,"−",Some("SF"),SubAlign::Center),
    k("7",117.5,384.6,40.0,30.5,18.6,KeyStyle::White,"7",Some("x↔y"),SubAlign::Center),
    k("8",190.5,384.6,40.0,30.5,18.6,KeyStyle::White,"8",Some("R▼"),SubAlign::Center),
    k("9",263.5,384.6,40.0,30.5,18.6,KeyStyle::White,"9",Some("R▲"),SubAlign::Center),
    k("plus",60.5,436.8,27.0,30.5,18.6,KeyStyle::Olive,"+",Some("CF"),SubAlign::Center),
    k("4",117.5,436.8,40.0,30.5,18.6,KeyStyle::White,"4",Some("1/x"),SubAlign::Center),
    k("5",190.5,436.8,40.0,30.5,18.6,KeyStyle::White,"5",Some("yˣ"),SubAlign::Center),
    k("6",263.5,436.8,40.0,30.5,18.6,KeyStyle::White,"6",Some("ABS"),SubAlign::Center),
    k("multiply",60.5,489.0,27.0,30.5,18.6,KeyStyle::Olive,"×",Some("F?"),SubAlign::Center),
    k("1",117.5,489.0,40.0,30.5,18.6,KeyStyle::White,"1",Some("PAUSE"),SubAlign::Center),
    k("2",190.5,489.0,40.0,30.5,18.6,KeyStyle::White,"2",Some("π"),SubAlign::Center),
    k("3",263.5,489.0,40.0,30.5,18.6,KeyStyle::White,"3",Some("REG"),SubAlign::Center),
    k("divide",60.5,541.2,27.0,30.5,18.6,KeyStyle::Olive,"÷",Some("N!"),SubAlign::Center),
    k("0",117.5,541.2,40.0,30.5,18.6,KeyStyle::White,"0",Some("LST x"),SubAlign::Center),
    k("decimal",190.5,541.2,40.0,30.5,18.6,KeyStyle::White,".",Some("H.MS+"),SubAlign::Center),
    k("rs",263.5,541.2,40.0,30.5,18.6,KeyStyle::White,"R/S",Some("SPACE"),SubAlign::Center),
];

pub struct KeyPressOverlay;

impl KeyPressOverlay {
    pub fn paint(ui: &Ui, host: Rect) {
        let scale = (host.width() / DESIGN_W).min(host.height() / DESIGN_H);
        if scale <= 0.0 { return; }
        let size = Vec2::new(DESIGN_W * scale, DESIGN_H * scale);
        let t = Transform {
            origin: Pos2::new(host.center().x - size.x * 0.5, host.center().y - size.y * 0.5),
            scale,
        };
        let pointer_pos = ui.input(|i| i.pointer.hover_pos());
        let pointer_down = ui.input(|i| i.pointer.primary_down());

        for key in KEYS {
            let x = key.cx - key.w * 0.5;
            let hit = t.rect(x - 2.0, key.y - 2.0, key.w + 4.0, key.h + 5.0);
            let target = pointer_down && pointer_pos.map(|pos| hit.contains(pos)).unwrap_or(false);
            let anim = ui.ctx().animate_bool_with_time(
                ui.make_persistent_id(("hp67-rigid-key-press", key.id)),
                target,
                0.050,
            );
            if anim > 0.001 {
                draw_rigid_pressed_key(ui.painter(), t, *key, anim);
            }
        }
    }
}

#[derive(Clone, Copy)]
struct Transform { origin: Pos2, scale: f32 }
impl Transform {
    fn s(self, v: f32) -> f32 { v * self.scale }
    fn pos(self, x: f32, y: f32) -> Pos2 { Pos2::new(self.origin.x + x*self.scale, self.origin.y + y*self.scale) }
    fn rect(self, x:f32,y:f32,w:f32,h:f32)->Rect { Rect::from_min_size(self.pos(x,y), Vec2::new(self.s(w),self.s(h))) }
}

fn draw_rigid_pressed_key(p: &Painter, t: Transform, key: KeySpec, press: f32) {
    let (face, front, side, text, subtext) = palette(key.style);
    let x = key.cx - key.w * 0.5;
    let travel = 2.8 * press;
    let y = key.y + travel;
    let skirt_top = y + key.top_h - 0.4;
    let skirt_h = key.h - key.top_h + 0.4;
    let bottom = y + key.h;

    // Erase the deforming key from the renderer underneath.  Keep this mask tight
    // so the external legends around the key remain untouched.
    p.rect_filled(t.rect(x - 2.2, key.y - 1.5, key.w + 4.4, key.h + 5.0), 0.0, PANEL);

    // A stationary dark socket makes the rigid cap look as if it is travelling
    // into the calculator instead of changing height.
    p.rect_filled(
        t.rect(x - 0.9, key.y + 0.5, key.w + 1.8, key.h + 2.0),
        t.s(2.1),
        Color32::from_rgb(19, 20, 19),
    );

    let shadow_alpha = (115.0 - 55.0 * press) as u8;
    p.rect_filled(
        t.rect(x + 1.5, bottom + 0.4, key.w - 1.0, 2.0),
        t.s(0.7),
        Color32::from_rgba_premultiplied(0, 0, 0, shadow_alpha),
    );

    p.rect_filled(
        t.rect(x - 0.7, y + 0.8, key.w + 1.4, key.top_h + 0.9),
        t.s(2.2),
        darker(side, 28),
    );

    let pressed_face = darker(face, (6.0 * press) as u8);
    let top = t.rect(x + 0.6, y, key.w - 1.2, key.top_h);
    p.rect_filled(top, t.s(2.0), pressed_face);
    p.rect_stroke(top, t.s(2.0), Stroke::new(t.s(0.62), Color32::from_rgba_premultiplied(0,0,0,62)));

    // The front face keeps exactly the same height and width throughout the
    // animation.  The complete keycap, including its legends, only translates.
    p.rect_filled(
        t.rect(x + 0.8, skirt_top, key.w - 1.6, skirt_h),
        t.s(0.8),
        darker(front, (5.0 * press) as u8),
    );
    p.rect_filled(t.rect(x + 0.8, skirt_top + 0.5, 1.15, skirt_h - 0.7), 0.0, side);
    p.rect_filled(t.rect(x + key.w - 1.95, skirt_top + 0.5, 1.15, skirt_h - 0.7), 0.0, darker(side,22));

    let hi_alpha = (150.0 - 55.0 * press) as u8;
    p.line_segment(
        [t.pos(x+2.3,y+1.05), t.pos(x+key.w-2.3,y+1.05)],
        Stroke::new(t.s(0.92), Color32::from_rgba_premultiplied(255,255,244,hi_alpha)),
    );
    p.line_segment(
        [t.pos(x+1.8,skirt_top+0.35), t.pos(x+key.w-1.8,skirt_top+0.35)],
        Stroke::new(t.s(0.66), Color32::from_rgba_premultiplied(255,255,238,100)),
    );

    draw_main(p, t, key, text, y + key.top_h * 0.47);
    if let Some(sub) = key.sub {
        draw_sub(p, t, key, sub, subtext, skirt_top + skirt_h * 0.55);
    }
}

fn draw_main(p: &Painter, t: Transform, key: KeySpec, color: Color32, cy: f32) {
    let size = match key.id {
        "f"|"g"|"h" => 13.4,
        "7"|"8"|"9"|"4"|"5"|"6"|"1"|"2"|"3"|"0" => 14.7,
        "minus"|"plus" => 15.2,
        "rs" => 11.8,
        "a"|"b"|"c"|"d"|"e" => 11.9,
        "enter" => 10.8,
        _ => 11.6,
    };
    if key.id == "enter" {
        bold_txt(p,t,key.cx-6.0,cy,size,color,"ENTER");
        triangle(p,t,key.cx+28.0,cy,3.1,color);
    } else {
        bold_txt(p,t,key.cx,cy,size,color,key.main);
    }
}

fn draw_sub(p: &Painter, t: Transform, key: KeySpec, sub: &str, color: Color32, cy: f32) {
    let size = if key.w >= 70.0 { 6.7 } else if sub.len() >= 5 { 6.0 } else { 6.6 };
    match key.sub_align {
        SubAlign::Center => bold_txt(p,t,key.cx,cy,size,color,sub),
        SubAlign::Right => bold_txt_aligned(p,t,key.cx+30.0,cy,size,color,sub,Align2::RIGHT_CENTER),
    }
}

fn palette(style: KeyStyle) -> (Color32, Color32, Color32, Color32, Color32) {
    match style {
        KeyStyle::Olive => (Color32::from_rgb(183,181,103),Color32::from_rgb(133,126,61),Color32::from_rgb(102,95,48),WHITE,DARK),
        KeyStyle::Orange => (Color32::from_rgb(246,181,43),Color32::from_rgb(197,126,18),Color32::from_rgb(145,84,7),DARK,DARK),
        KeyStyle::Blue => (Color32::from_rgb(69,194,220),Color32::from_rgb(31,137,162),Color32::from_rgb(18,92,109),DARK,DARK),
        KeyStyle::White => (Color32::from_rgb(236,237,232),Color32::from_rgb(192,196,193),Color32::from_rgb(148,153,151),DARK,DARK),
        KeyStyle::Black => (Color32::from_rgb(22,24,25),Color32::from_rgb(10,11,12),Color32::from_rgb(4,5,5),WHITE,WHITE),
    }
}

fn bold_txt(p:&Painter,t:Transform,x:f32,y:f32,size:f32,color:Color32,s:&str) {
    bold_txt_aligned(p,t,x,y,size,color,s,Align2::CENTER_CENTER)
}

fn bold_txt_aligned(p:&Painter,t:Transform,x:f32,y:f32,size:f32,color:Color32,s:&str,align:Align2) {
    let font = FontId::new(t.s(size), FontFamily::Proportional);
    let pos = t.pos(x,y);
    p.text(pos,align,s,font.clone(),color);
    p.text(Pos2::new(pos.x+t.s(0.28),pos.y),align,s,font,color);
}

fn triangle(p:&Painter,t:Transform,cx:f32,cy:f32,h:f32,c:Color32) {
    p.add(eframe::egui::Shape::convex_polygon(
        vec![t.pos(cx,cy-h),t.pos(cx-h,cy+h*0.72),t.pos(cx+h,cy+h*0.72)],
        c,
        Stroke::NONE,
    ));
}

fn darker(c:Color32,n:u8)->Color32 {
    Color32::from_rgb(c.r().saturating_sub(n),c.g().saturating_sub(n),c.b().saturating_sub(n))
}
