use eframe::egui::{Align2, Color32, FontId, Painter, Pos2, Rect, Shape, Stroke, Vec2};

const DESIGN_W: f32 = 330.0;
const DESIGN_H: f32 = 620.0;

const PANEL: Color32 = Color32::from_rgb(45, 46, 43);
const WHITE: Color32 = Color32::from_rgb(232, 234, 229);
const F_YELLOW: Color32 = Color32::from_rgb(218, 207, 55);
const G_CYAN: Color32 = Color32::from_rgb(80, 202, 222);

#[derive(Clone, Copy)]
enum KeyStyle { Olive, Orange, Blue, White, Black }

#[derive(Clone, Copy)]
struct KeySpec {
    id: &'static str,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    main: &'static str,
    sub: Option<&'static str>,
    style: KeyStyle,
}

const fn k(
    id: &'static str, x: f32, y: f32, w: f32, h: f32,
    main: &'static str, sub: Option<&'static str>, style: KeyStyle,
) -> KeySpec {
    KeySpec { id, x, y, w, h, main, sub, style }
}

// Measured from the supplied HP-67 reference.
// Source photo key boxes (pixels) were normalized against the inner black face
// to the 330x620 design space. Important consequence: the lower keyboard is
// NOT a uniform square grid. Operator keys are narrow; numeric keys are wide.
const KEYS: &[KeySpec] = &[
    // A-E: measured x ~= [48.5, 98.3, 148.9, 200.4, 251.0], w ~= 30-31.
    k("a",49.0,170.0,30.0,26.0,"A",None,KeyStyle::Olive),
    k("b",99.0,170.0,31.0,26.0,"B",None,KeyStyle::Olive),
    k("c",149.0,170.0,31.0,26.0,"C",None,KeyStyle::Olive),
    k("d",200.0,170.0,31.0,26.0,"D",None,KeyStyle::Olive),
    k("e",251.0,170.0,31.0,26.0,"E",None,KeyStyle::Olive),

    k("sigma",49.0,224.0,31.0,26.0,"",Some(""),KeyStyle::Olive),
    k("gto",99.0,224.0,31.0,26.0,"GTO",Some("RTN"),KeyStyle::Olive),
    k("dsp",149.0,224.0,31.0,26.0,"DSP",Some("ENG"),KeyStyle::Olive),
    k("indirect",200.0,224.0,31.0,26.0,"(i)",Some(""),KeyStyle::Olive),
    k("sst",251.0,224.0,31.0,26.0,"SST",Some("BST"),KeyStyle::Olive),

    k("f",48.0,279.0,32.0,25.0,"f",None,KeyStyle::Orange),
    k("g",99.0,279.0,31.0,25.0,"g",None,KeyStyle::Blue),
    k("sto",149.0,279.0,31.0,26.0,"STO",Some("ST I"),KeyStyle::Olive),
    k("rcl",200.0,279.0,31.0,26.0,"RCL",Some("RC I"),KeyStyle::Olive),
    k("h",251.0,279.0,31.0,25.0,"h",None,KeyStyle::Black),

    k("enter",49.0,333.0,80.0,27.0,"ENTER",Some("DEG"),KeyStyle::Olive),
    k("chs",150.0,333.0,30.0,27.0,"CHS",Some("RAD"),KeyStyle::Olive),
    k("eex",200.0,333.0,31.0,27.0,"EEX",Some("GRD"),KeyStyle::Olive),
    k("clx",251.0,333.0,31.0,27.0,"CLx",Some("DEL"),KeyStyle::Olive),

    // Lower matrix measured directly from the HP-67 reference:
    // operator x/w ~= 49/23; number columns ~= 99/37, 172/37, 245/38.
    k("minus",49.0,385.0,23.0,27.0,"-",Some("SF"),KeyStyle::Olive),
    k("7",99.0,385.0,37.0,27.0,"7",Some(""),KeyStyle::White),
    k("8",172.0,385.0,37.0,27.0,"8",Some(""),KeyStyle::White),
    k("9",245.0,385.0,38.0,27.0,"9",Some(""),KeyStyle::White),

    k("plus",49.0,437.0,23.0,27.0,"+",Some("CF"),KeyStyle::Olive),
    k("4",99.0,437.0,37.0,27.0,"4",Some(""),KeyStyle::White),
    k("5",172.0,437.0,37.0,27.0,"5",Some(""),KeyStyle::White),
    k("6",245.0,437.0,38.0,27.0,"6",Some("ABS"),KeyStyle::White),

    k("multiply",49.0,489.0,23.0,27.0,"",Some("F?"),KeyStyle::Olive),
    k("1",99.0,489.0,37.0,27.0,"1",Some("PAUSE"),KeyStyle::White),
    k("2",172.0,489.0,37.0,27.0,"2",Some(""),KeyStyle::White),
    k("3",245.0,489.0,38.0,27.0,"3",Some("REG"),KeyStyle::White),

    k("divide",49.0,541.0,23.0,27.0,"",Some("N!"),KeyStyle::Olive),
    k("0",99.0,541.0,37.0,28.0,"0",Some("LST x"),KeyStyle::White),
    k("decimal",172.0,541.0,37.0,28.0,".",Some("H.MS+"),KeyStyle::White),
    k("rs",245.0,541.0,38.0,28.0,"R/S",Some("SPACE"),KeyStyle::White),
];

pub struct KeyDepthOverlay;

impl KeyDepthOverlay {
    pub fn paint(p: &Painter, host: Rect) {
        let scale = (host.width() / DESIGN_W).min(host.height() / DESIGN_H);
        if scale <= 0.0 { return; }
        let size = Vec2::new(DESIGN_W * scale, DESIGN_H * scale);
        let t = Transform {
            origin: Pos2::new(host.center().x - size.x * 0.5, host.center().y - size.y * 0.5),
            scale,
        };

        // Replace the legacy keyboard completely. This lets the measured matrix
        // own both geometry and labels instead of stacking corrections on top.
        p.rect_filled(t.rect(38.0, 137.0, 254.0, 450.0), 0.0, PANEL);

        draw_legends(p, t);
        for key in KEYS { draw_key(p, t, *key); }
    }
}

#[derive(Clone, Copy)]
struct Transform { origin: Pos2, scale: f32 }
impl Transform {
    fn s(self, v: f32) -> f32 { v * self.scale }
    fn pos(self, x: f32, y: f32) -> Pos2 {
        Pos2::new(self.origin.x + x*self.scale, self.origin.y + y*self.scale)
    }
    fn rect(self, x: f32, y: f32, w: f32, h: f32) -> Rect {
        Rect::from_min_size(self.pos(x,y), Vec2::new(self.s(w), self.s(h)))
    }
}

fn palette(style: KeyStyle) -> (Color32, Color32, Color32, Color32) {
    match style {
        KeyStyle::Olive => (
            Color32::from_rgb(170,170,91),
            Color32::from_rgb(128,129,66),
            Color32::from_rgb(93,94,48),
            WHITE,
        ),
        KeyStyle::Orange => (
            Color32::from_rgb(238,170,35),
            Color32::from_rgb(193,124,19),
            Color32::from_rgb(142,83,8),
            Color32::from_rgb(32,30,22),
        ),
        KeyStyle::Blue => (
            Color32::from_rgb(53,181,208),
            Color32::from_rgb(31,133,157),
            Color32::from_rgb(19,91,108),
            Color32::from_rgb(20,30,33),
        ),
        KeyStyle::White => (
            Color32::from_rgb(230,232,225),
            Color32::from_rgb(190,194,190),
            Color32::from_rgb(148,153,151),
            Color32::from_rgb(23,27,29),
        ),
        KeyStyle::Black => (
            Color32::from_rgb(25,28,29),
            Color32::from_rgb(10,12,13),
            Color32::from_rgb(4,5,5),
            WHITE,
        ),
    }
}

fn draw_key(p: &Painter, t: Transform, key: KeySpec) {
    let (face, front, side, text) = palette(key.style);

    let skirt_h = if key.sub.is_some() { 8.4 } else { 6.0 };
    let skirt_top = key.y + key.h - skirt_h;
    let bottom = key.y + key.h;
    let top_face_h = skirt_top - key.y;

    // Tight contact shadow, not a fake lower rim.
    p.rect_filled(
        t.rect(key.x + 1.2, key.y + 2.2, key.w, key.h + 1.6),
        t.s(2.1),
        Color32::from_rgba_premultiplied(0,0,0,125),
    );

    // Top face is intentionally rectangular, matching the real HP-67.
    let top = t.rect(key.x, key.y, key.w, top_face_h + 1.0);
    p.rect_filled(top, t.s(2.2), face);
    p.line_segment(
        [t.pos(key.x + 1.8,key.y + 1.0), t.pos(key.x + key.w - 1.8,key.y + 1.0)],
        Stroke::new(t.s(0.8), Color32::from_rgba_premultiplied(255,255,245,130)),
    );

    // Sloping front skirt. No lower highlight/ridge: the supplied HP-67 has a
    // clean front plane that dies into the chassis shadow.
    let top_inset = 0.9;
    let bottom_inset = if key.w >= 70.0 { 4.4 } else if key.w <= 24.0 { 2.1 } else { 3.0 };
    let tl = t.pos(key.x + top_inset, skirt_top);
    let tr = t.pos(key.x + key.w - top_inset, skirt_top);
    let bl = t.pos(key.x + bottom_inset, bottom);
    let br = t.pos(key.x + key.w - bottom_inset, bottom);

    p.add(Shape::convex_polygon(
        vec![tl, t.pos(key.x+top_inset+1.6,skirt_top+0.8), t.pos(key.x+bottom_inset+1.5,bottom-0.7), bl],
        side, Stroke::NONE,
    ));
    p.add(Shape::convex_polygon(
        vec![t.pos(key.x+key.w-top_inset-1.6,skirt_top+0.8), tr, br, t.pos(key.x+key.w-bottom_inset-1.5,bottom-0.7)],
        Color32::from_rgb(
            side.r().saturating_sub(18),
            side.g().saturating_sub(18),
            side.b().saturating_sub(18),
        ),
        Stroke::NONE,
    ));
    p.add(Shape::convex_polygon(vec![tl,tr,br,bl], front, Stroke::NONE));

    // Only the bend line at the top of the skirt.
    p.line_segment(
        [t.pos(key.x+top_inset+1.0,skirt_top+0.45), t.pos(key.x+key.w-top_inset-1.0,skirt_top+0.45)],
        Stroke::new(t.s(0.65), Color32::from_rgba_premultiplied(255,255,240,105)),
    );

    draw_main(p,t,key,text,key.y + top_face_h*0.48);
    if key.sub.is_some() {
        draw_sub(p,t,key,text,skirt_top + skirt_h*0.53);
    }
}

fn draw_main(p:&Painter,t:Transform,key:KeySpec,color:Color32,cy:f32) {
    let cx = key.x + key.w*0.5;
    match key.id {
        "sigma" => {
            sigma(p,t,cx-2.1,cy,8.4,color);
            txt(p,t,cx+4.1,cy,8.9,color,"+");
        }
        "multiply" => cross(p,t,cx,cy,3.5,color),
        "divide" => divide(p,t,cx,cy,3.5,color),
        "enter" => {
            txt(p,t,cx-5.0,cy,10.1,color,"ENTER");
            triangle(p,t,cx+24.5,cy,2.8,true,color);
        }
        _ => {
            let size = match key.id {
                "f"|"g"|"h" => 11.8,
                "7"|"8"|"9"|"4"|"5"|"6"|"1"|"2"|"3"|"0" => 12.8,
                "minus"|"plus" => 13.4,
                "rs" => 10.2,
                _ => 10.3,
            };
            txt(p,t,cx,cy,size,color,key.main);
        }
    }
}

fn draw_sub(p:&Painter,t:Transform,key:KeySpec,color:Color32,cy:f32) {
    let cx=key.x+key.w*0.5;
    match key.id {
        "sigma" => { sigma(p,t,cx-1.8,cy,5.7,color); txt(p,t,cx+3.4,cy,6.0,color,"-"); }
        "indirect" => swap(p,t,cx,cy,"x","I",6.0,color),
        "7" => swap(p,t,cx,cy,"x","y",6.1,color),
        "8" => r_arrow(p,t,cx,cy,false,6.0,color),
        "9" => r_arrow(p,t,cx,cy,true,6.0,color),
        "4" => one_over_x(p,t,cx,cy,6.0,color),
        "5" => power(p,t,cx,cy,"y","x",6.0,color,color),
        "2" => pi(p,t,cx,cy,6.4,color),
        _ => {
            let sub = key.sub.unwrap_or("");
            let size = if key.w >= 70.0 { 6.0 } else if sub.len() >= 5 { 5.5 } else { 6.0 };
            txt(p,t,cx,cy,size,color,sub);
        }
    }
}

fn draw_legends(p:&Painter,t:Transform) {
    // Top mathematical row: optical centres are the measured key centres.
    one_over_x(p,t,64.0,151.5,9.6,WHITE);
    sqrt_x(p,t,114.5,151.5,9.6,WHITE);
    power(p,t,164.5,151.5,"y","x",9.6,WHITE,WHITE);
    r_arrow(p,t,215.5,151.5,false,9.6,WHITE);
    swap(p,t,266.5,151.5,"x","y",9.3,WHITE);

    for (x,v) in [(64.0,"a"),(114.5,"b"),(164.5,"c"),(215.5,"d"),(266.5,"e")] {
        txt(p,t,x,205.5,7.8,F_YELLOW,v);
    }

    x_bar(p,t,57.5,262.5,7.4,F_YELLOW); txt(p,t,70.5,262.5,7.6,G_CYAN,"s");
    txt(p,t,107.0,262.5,7.7,F_YELLOW,"GSB"); txt(p,t,122.0,262.5,7.7,G_CYAN,"f");
    txt(p,t,157.0,262.5,7.7,F_YELLOW,"FIX"); txt(p,t,175.0,262.5,7.7,G_CYAN,"SCI");
    txt(p,t,215.5,262.5,7.7,F_YELLOW,"RND");
    txt(p,t,258.0,262.5,7.7,F_YELLOW,"LBL"); txt(p,t,275.0,262.5,7.7,G_CYAN,"f");

    txt(p,t,157.5,316.5,7.6,F_YELLOW,"DSZ"); txt(p,t,177.0,316.5,7.6,G_CYAN,"(i)");
    txt(p,t,208.5,316.5,7.6,F_YELLOW,"ISZ"); txt(p,t,228.0,316.5,7.6,G_CYAN,"(i)");

    txt(p,t,63.0,372.8,7.6,F_YELLOW,"W/DATA"); txt(p,t,113.5,372.8,7.6,G_CYAN,"MERGE");
    swap(p,t,164.5,372.8,"P","S",7.4,F_YELLOW);
    txt(p,t,215.5,372.8,7.6,F_YELLOW,"CL REG"); txt(p,t,266.5,372.8,7.6,F_YELLOW,"CL PRGM");

    eq_pair(p,t,60.5,424.5,false);
    txt(p,t,108.5,424.5,7.6,F_YELLOW,"LN"); power(p,t,127.0,424.5,"e","x",7.6,G_CYAN,G_CYAN);
    txt(p,t,181.0,424.5,7.6,F_YELLOW,"LOG"); power(p,t,205.0,424.5,"10","x",7.6,G_CYAN,G_CYAN);
    sqrt_x(p,t,254.5,424.5,7.6,F_YELLOW); power(p,t,276.0,424.5,"x","2",7.6,G_CYAN,G_CYAN);

    eq_pair(p,t,60.5,476.5,true);
    inverse(p,t,117.5,476.5,"SIN"); inverse(p,t,190.5,476.5,"COS"); inverse(p,t,264.0,476.5,"TAN");

    rel_pair(p,t,60.5,528.5,'<',true,7.2);
    swap_two_color(p,t,117.5,528.5,"R","P",7.4);
    swap_two_color(p,t,190.5,528.5,"D","R",7.4);
    swap_two_color(p,t,264.0,528.5,"H","H.MS",7.1);

    rel_pair(p,t,60.5,580.0,'>',false,7.2);
    txt(p,t,106.0,580.0,7.4,F_YELLOW,"%"); txt(p,t,126.0,580.0,7.4,G_CYAN,"%CH");
    txt(p,t,179.0,580.0,7.4,F_YELLOW,"INT"); txt(p,t,204.0,580.0,7.4,G_CYAN,"FRAC");
    txt(p,t,254.0,580.0,7.4,F_YELLOW,"-x-"); txt(p,t,279.0,580.0,7.4,G_CYAN,"STK");
}

fn txt(p:&Painter,t:Transform,x:f32,y:f32,size:f32,c:Color32,s:&str) {
    p.text(t.pos(x,y),Align2::CENTER_CENTER,s,FontId::proportional(t.s(size)),c);
}

fn one_over_x(p:&Painter,t:Transform,cx:f32,cy:f32,size:f32,c:Color32) {
    let s=size/9.0;
    txt(p,t,cx-5.7*s,cy,size,c,"1");
    p.line_segment([t.pos(cx-1.7*s,cy+4.1*s),t.pos(cx+2.0*s,cy-4.1*s)],Stroke::new(t.s(0.8*s),c));
    txt(p,t,cx+5.7*s,cy,size,c,"x");
}
fn sqrt_x(p:&Painter,t:Transform,cx:f32,cy:f32,size:f32,c:Color32) {
    let s=size/9.0; let x=cx-8.0*s;
    p.line_segment([t.pos(x,cy+0.6*s),t.pos(x+2.3*s,cy+4.0*s)],Stroke::new(t.s(0.9*s),c));
    p.line_segment([t.pos(x+2.3*s,cy+4.0*s),t.pos(x+5.3*s,cy-4.8*s)],Stroke::new(t.s(0.9*s),c));
    p.line_segment([t.pos(x+5.3*s,cy-4.8*s),t.pos(x+14.3*s,cy-4.8*s)],Stroke::new(t.s(0.75*s),c));
    txt(p,t,cx+3.2*s,cy+0.4*s,size,c,"x");
}
fn power(p:&Painter,t:Transform,cx:f32,cy:f32,b:&str,e:&str,size:f32,bc:Color32,ec:Color32) {
    let off=if b.len()>1{5.0}else{2.6};
    txt(p,t,cx-off,cy+1.0,size,bc,b);
    txt(p,t,cx+off+2.0,cy-size*0.42,size*0.63,ec,e);
}
fn r_arrow(p:&Painter,t:Transform,cx:f32,cy:f32,up:bool,size:f32,c:Color32) {
    txt(p,t,cx-3.2,cy,size,c,"R");
    triangle(p,t,cx+5.2,cy,size*0.28,up,c);
}
fn swap(p:&Painter,t:Transform,cx:f32,cy:f32,l:&str,r:&str,size:f32,c:Color32) {
    let span=if r.len()>1{9.5}else{7.6};
    txt(p,t,cx-span,cy,size,c,l); txt(p,t,cx+span,cy,size,c,r);
    double_arrow(p,t,cx,cy,size*0.48,c);
}
fn swap_two_color(p:&Painter,t:Transform,cx:f32,cy:f32,l:&str,r:&str,size:f32) {
    let span=if r.len()>1{10.5}else{8.0};
    txt(p,t,cx-span,cy,size,F_YELLOW,l); txt(p,t,cx+span,cy,size,G_CYAN,r);
    double_arrow(p,t,cx,cy,size*0.48,G_CYAN);
}
fn double_arrow(p:&Painter,t:Transform,cx:f32,cy:f32,h:f32,c:Color32) {
    let x1=cx-h; let x2=cx+h; let yu=cy-1.35; let yd=cy+1.35;
    p.line_segment([t.pos(x1,yu),t.pos(x2,yu)],Stroke::new(t.s(0.65),c));
    p.line_segment([t.pos(x2-1.8,yu-1.3),t.pos(x2,yu)],Stroke::new(t.s(0.65),c));
    p.line_segment([t.pos(x2-1.8,yu+1.3),t.pos(x2,yu)],Stroke::new(t.s(0.65),c));
    p.line_segment([t.pos(x2,yd),t.pos(x1,yd)],Stroke::new(t.s(0.65),c));
    p.line_segment([t.pos(x1+1.8,yd-1.3),t.pos(x1,yd)],Stroke::new(t.s(0.65),c));
    p.line_segment([t.pos(x1+1.8,yd+1.3),t.pos(x1,yd)],Stroke::new(t.s(0.65),c));
}
fn triangle(p:&Painter,t:Transform,cx:f32,cy:f32,h:f32,up:bool,c:Color32) {
    let d=if up{-1.0}else{1.0};
    p.add(Shape::convex_polygon(vec![t.pos(cx,cy+d*h),t.pos(cx-h,cy-d*h*0.7),t.pos(cx+h,cy-d*h*0.7)],c,Stroke::NONE));
}
fn sigma(p:&Painter,t:Transform,cx:f32,cy:f32,size:f32,c:Color32) {
    let s=size/8.0; let l=cx-4.0*s; let r=cx+4.0*s; let top=cy-4.0*s; let bot=cy+4.0*s;
    p.line_segment([t.pos(l,top),t.pos(r,top)],Stroke::new(t.s(0.85*s),c));
    p.line_segment([t.pos(l,top),t.pos(cx+0.8*s,cy)],Stroke::new(t.s(0.85*s),c));
    p.line_segment([t.pos(cx+0.8*s,cy),t.pos(l,bot)],Stroke::new(t.s(0.85*s),c));
    p.line_segment([t.pos(l,bot),t.pos(r,bot)],Stroke::new(t.s(0.85*s),c));
}
fn cross(p:&Painter,t:Transform,cx:f32,cy:f32,h:f32,c:Color32) {
    p.line_segment([t.pos(cx-h,cy-h),t.pos(cx+h,cy+h)],Stroke::new(t.s(0.9),c));
    p.line_segment([t.pos(cx-h,cy+h),t.pos(cx+h,cy-h)],Stroke::new(t.s(0.9),c));
}
fn divide(p:&Painter,t:Transform,cx:f32,cy:f32,h:f32,c:Color32) {
    p.line_segment([t.pos(cx-h,cy),t.pos(cx+h,cy)],Stroke::new(t.s(0.9),c));
    p.circle_filled(t.pos(cx,cy-3.7),t.s(0.8),c); p.circle_filled(t.pos(cx,cy+3.7),t.s(0.8),c);
}
fn pi(p:&Painter,t:Transform,cx:f32,cy:f32,size:f32,c:Color32) {
    let s=size/6.4;
    p.line_segment([t.pos(cx-4.0*s,cy-2.5*s),t.pos(cx+4.0*s,cy-2.5*s)],Stroke::new(t.s(0.7*s),c));
    p.line_segment([t.pos(cx-2.0*s,cy-2.5*s),t.pos(cx-2.0*s,cy+2.6*s)],Stroke::new(t.s(0.7*s),c));
    p.line_segment([t.pos(cx+2.0*s,cy-2.5*s),t.pos(cx+2.0*s,cy+2.6*s)],Stroke::new(t.s(0.7*s),c));
}
fn x_bar(p:&Painter,t:Transform,cx:f32,cy:f32,size:f32,c:Color32) {
    txt(p,t,cx,cy+0.5,size,c,"x");
    p.line_segment([t.pos(cx-3.0,cy-4.2),t.pos(cx+3.0,cy-4.2)],Stroke::new(t.s(0.75),c));
}
fn eq_pair(p:&Painter,t:Transform,cx:f32,cy:f32,ne:bool) {
    txt(p,t,cx-13.0,cy,7.2,F_YELLOW,"x");
    if ne { not_equal(p,t,cx-5.0,cy,F_YELLOW); } else { txt(p,t,cx-5.0,cy,7.2,F_YELLOW,"="); }
    txt(p,t,cx+1.0,cy,7.2,F_YELLOW,"0");
    txt(p,t,cx+7.0,cy,7.2,G_CYAN,"x");
    if ne { not_equal(p,t,cx+15.0,cy,G_CYAN); } else { txt(p,t,cx+15.0,cy,7.2,G_CYAN,"="); }
    txt(p,t,cx+22.0,cy,7.2,G_CYAN,"y");
}
fn not_equal(p:&Painter,t:Transform,cx:f32,cy:f32,c:Color32) {
    p.line_segment([t.pos(cx-2.7,cy-1.5),t.pos(cx+2.7,cy-1.5)],Stroke::new(t.s(0.7),c));
    p.line_segment([t.pos(cx-2.7,cy+1.5),t.pos(cx+2.7,cy+1.5)],Stroke::new(t.s(0.7),c));
    p.line_segment([t.pos(cx-2.1,cy+3.1),t.pos(cx+2.1,cy-3.1)],Stroke::new(t.s(0.7),c));
}
fn inverse(p:&Painter,t:Transform,cx:f32,cy:f32,name:&str) {
    txt(p,t,cx-2.0,cy,7.4,F_YELLOW,name);
    txt(p,t,cx+13.0,cy-4.1,5.0,G_CYAN,"-1");
}
fn rel_pair(p:&Painter,t:Transform,cx:f32,cy:f32,op:char,inclusive:bool,size:f32) {
    let s=if op=='<'{"<"}else{">"};
    txt(p,t,cx-13.0,cy,size,F_YELLOW,"x"); txt(p,t,cx-6.5,cy,size,F_YELLOW,s); txt(p,t,cx,cy,size,F_YELLOW,"0");
    txt(p,t,cx+7.0,cy,size,G_CYAN,"x");
    if inclusive { rel_equal(p,t,cx+14.5,cy,op,G_CYAN); } else { txt(p,t,cx+14.5,cy,size,G_CYAN,s); }
    txt(p,t,cx+22.0,cy,size,G_CYAN,"y");
}
fn rel_equal(p:&Painter,t:Transform,cx:f32,cy:f32,op:char,c:Color32) {
    let flip=if op=='<'{1.0}else{-1.0};
    p.line_segment([t.pos(cx+2.4*flip,cy-2.8),t.pos(cx-2.0*flip,cy)],Stroke::new(t.s(0.75),c));
    p.line_segment([t.pos(cx-2.0*flip,cy),t.pos(cx+2.4*flip,cy+2.8)],Stroke::new(t.s(0.75),c));
    p.line_segment([t.pos(cx-2.6,cy+4.0),t.pos(cx+2.6,cy+4.0)],Stroke::new(t.s(0.75),c));
}
