use eframe::egui::{Align2, Color32, FontFamily, FontId, Painter, Pos2, Rect, Shape, Stroke, Ui, Vec2};

const DESIGN_W: f32 = 330.0;
const DESIGN_H: f32 = 620.0;

const PANEL: Color32 = Color32::from_rgb(45, 46, 43);
const WHITE: Color32 = Color32::from_rgb(238, 239, 232);
const YELLOW: Color32 = Color32::from_rgb(220, 208, 54);
const CYAN: Color32 = Color32::from_rgb(82, 207, 225);
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

const fn k(
    id: &'static str, cx: f32, y: f32, w: f32, h: f32, top_h: f32,
    style: KeyStyle, main: &'static str, sub: Option<&'static str>, sub_align: SubAlign,
) -> KeySpec {
    KeySpec { id, cx, y, w, h, top_h, style, main, sub, sub_align }
}

const KEYS: &[KeySpec] = &[
    k("a", 65.0, 169.5, 35.0, 29.5, 20.0, KeyStyle::Olive, "A", None, SubAlign::Center),
    k("b",116.0, 169.5, 35.0, 29.5, 20.0, KeyStyle::Olive, "B", None, SubAlign::Center),
    k("c",167.0, 169.5, 35.0, 29.5, 20.0, KeyStyle::Olive, "C", None, SubAlign::Center),
    k("d",218.0, 169.5, 35.0, 29.5, 20.0, KeyStyle::Olive, "D", None, SubAlign::Center),
    k("e",269.0, 169.5, 35.0, 29.5, 20.0, KeyStyle::Olive, "E", None, SubAlign::Center),

    k("sigma",65.0,223.2,35.0,31.0,19.1,KeyStyle::Olive,"",Some(""),SubAlign::Center),
    k("gto",116.0,223.2,35.0,31.0,19.1,KeyStyle::Olive,"GTO",Some("RTN"),SubAlign::Center),
    k("dsp",167.0,223.2,35.0,31.0,19.1,KeyStyle::Olive,"DSP",Some("ENG"),SubAlign::Center),
    k("indirect",218.0,223.2,35.0,31.0,19.1,KeyStyle::Olive,"(i)",Some(""),SubAlign::Center),
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
    k("7",117.5,384.6,40.0,30.5,18.6,KeyStyle::White,"7",Some(""),SubAlign::Center),
    k("8",190.5,384.6,40.0,30.5,18.6,KeyStyle::White,"8",Some(""),SubAlign::Center),
    k("9",263.5,384.6,40.0,30.5,18.6,KeyStyle::White,"9",Some(""),SubAlign::Center),

    k("plus",60.5,436.8,27.0,30.5,18.6,KeyStyle::Olive,"+",Some("CF"),SubAlign::Center),
    k("4",117.5,436.8,40.0,30.5,18.6,KeyStyle::White,"4",Some(""),SubAlign::Center),
    k("5",190.5,436.8,40.0,30.5,18.6,KeyStyle::White,"5",Some(""),SubAlign::Center),
    k("6",263.5,436.8,40.0,30.5,18.6,KeyStyle::White,"6",Some("ABS"),SubAlign::Center),

    k("multiply",60.5,489.0,27.0,30.5,18.6,KeyStyle::Olive,"",Some("F?"),SubAlign::Center),
    k("1",117.5,489.0,40.0,30.5,18.6,KeyStyle::White,"1",Some("PAUSE"),SubAlign::Center),
    k("2",190.5,489.0,40.0,30.5,18.6,KeyStyle::White,"2",Some(""),SubAlign::Center),
    k("3",263.5,489.0,40.0,30.5,18.6,KeyStyle::White,"3",Some("REG"),SubAlign::Center),

    k("divide",60.5,541.2,27.0,30.5,18.6,KeyStyle::Olive,"",Some("N!"),SubAlign::Center),
    k("0",117.5,541.2,40.0,30.5,18.6,KeyStyle::White,"0",Some("LST x"),SubAlign::Center),
    k("decimal",190.5,541.2,40.0,30.5,18.6,KeyStyle::White,".",Some("H.MS+"),SubAlign::Center),
    k("rs",263.5,541.2,40.0,30.5,18.6,KeyStyle::White,"R/S",Some("SPACE"),SubAlign::Center),
];

pub struct KeyDepthOverlay;

impl KeyDepthOverlay {
    pub fn paint(ui: &Ui, host: Rect) {
        let scale = (host.width() / DESIGN_W).min(host.height() / DESIGN_H);
        if scale <= 0.0 { return; }

        let size = Vec2::new(DESIGN_W * scale, DESIGN_H * scale);
        let t = Transform {
            origin: Pos2::new(host.center().x - size.x * 0.5, host.center().y - size.y * 0.5),
            scale,
        };
        let p = ui.painter();

        // Own the complete keyboard region so no remnants of the legacy key renderer remain.
        p.rect_filled(t.rect(38.0, 137.0, 254.0, 449.5), 0.0, PANEL);
        draw_legends(p, t);

        let pointer_pos = ui.input(|i| i.pointer.hover_pos());
        let pointer_down = ui.input(|i| i.pointer.primary_down());

        for key in KEYS {
            let x = key.cx - key.w * 0.5;
            let hit = t.rect(x - 2.0, key.y - 2.0, key.w + 4.0, key.h + 5.0);
            let target = pointer_down && pointer_pos.map(|pos| hit.contains(pos)).unwrap_or(false);
            let anim = ui.ctx().animate_bool_with_time(
                ui.make_persistent_id(("hp67-key-press", key.id)),
                target,
                0.055,
            );
            draw_key(p, t, *key, anim);
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

fn draw_key(p:&Painter,t:Transform,key:KeySpec,press:f32) {
    let (face, front, side, text, subtext) = palette(key.style);
    let x = key.cx - key.w*0.5;

    // A real HP-67 keycap reads as rectangular from the front. The depth comes
    // from the darker sloping face, not from tapering the outer silhouette.
    let travel = 2.8 * press;
    let top_y = key.y + travel;
    let skirt_top = top_y + key.top_h - 0.4;
    let bottom = key.y + key.h;
    let skirt_h = (bottom - skirt_top).max(2.2);

    // Fixed contact shadow; it tightens as the key sinks.
    let shadow_alpha = (130.0 - 55.0 * press) as u8;
    p.rect_filled(
        t.rect(x + 1.5, key.y + key.h + 0.5, key.w - 1.0, 2.3 - 1.1*press),
        t.s(0.8),
        Color32::from_rgba_premultiplied(0,0,0,shadow_alpha),
    );

    // Tiny dark shell around the cap. No diagonals, no clipped corners.
    p.rect_filled(
        t.rect(x - 0.7, top_y + 0.8, key.w + 1.4, key.top_h + 0.9),
        t.s(2.2),
        darker(side, 28),
    );

    let pressed_face = darker(face, (8.0 * press) as u8);
    let top = t.rect(x + 0.6, top_y, key.w - 1.2, key.top_h);
    p.rect_filled(top, t.s(2.0), pressed_face);
    p.rect_stroke(top, t.s(2.0), Stroke::new(t.s(0.62), Color32::from_rgba_premultiplied(0,0,0,62)));

    // Rectangular sloping front plane. Same width top and bottom: this removes
    // the trapezoidal/cut-off look while preserving the HP-67 3D lip.
    let front_rect = t.rect(x + 0.8, skirt_top, key.w - 1.6, skirt_h);
    p.rect_filled(front_rect, t.s(0.8), darker(front, (7.0 * press) as u8));

    // Narrow vertical side facets only; they do not change the silhouette.
    p.rect_filled(t.rect(x + 0.8, skirt_top + 0.5, 1.15, (skirt_h - 0.7).max(0.8)), 0.0, side);
    p.rect_filled(t.rect(x + key.w - 1.95, skirt_top + 0.5, 1.15, (skirt_h - 0.7).max(0.8)), 0.0, darker(side,22));

    // Highlights fade while depressed.
    let hi_alpha = (150.0 - 65.0 * press) as u8;
    p.line_segment(
        [t.pos(x+2.3,top_y+1.05),t.pos(x+key.w-2.3,top_y+1.05)],
        Stroke::new(t.s(0.92),Color32::from_rgba_premultiplied(255,255,244,hi_alpha)),
    );
    p.line_segment(
        [t.pos(x+1.8,skirt_top+0.35),t.pos(x+key.w-1.8,skirt_top+0.35)],
        Stroke::new(t.s(0.66),Color32::from_rgba_premultiplied(255,255,238,100)),
    );

    draw_main(p,t,key,text,top_y+key.top_h*0.47);
    if key.sub.is_some() {
        draw_sub(p,t,key,subtext,skirt_top+skirt_h*0.55);
    }
}

fn draw_main(p:&Painter,t:Transform,key:KeySpec,color:Color32,cy:f32) {
    match key.id {
        "sigma" => { sigma(p,t,key.cx-2.4,cy,9.7,color); bold_txt(p,t,key.cx+5.0,cy,10.1,color,"+"); }
        "multiply" => cross(p,t,key.cx,cy,4.3,color),
        "divide" => divide(p,t,key.cx,cy,4.3,color),
        "enter" => { bold_txt(p,t,key.cx-6.0,cy,10.8,color,"ENTER"); triangle(p,t,key.cx+28.0,cy,3.1,true,color); }
        _ => {
            let size = match key.id {
                "f"|"g"|"h" => 13.4,
                "7"|"8"|"9"|"4"|"5"|"6"|"1"|"2"|"3"|"0" => 14.7,
                "minus"|"plus" => 15.2,
                "rs" => 11.8,
                "a"|"b"|"c"|"d"|"e" => 11.9,
                _ => 11.6,
            };
            bold_txt(p,t,key.cx,cy,size,color,key.main);
        }
    }
}

fn draw_sub(p:&Painter,t:Transform,key:KeySpec,color:Color32,cy:f32) {
    let sub=key.sub.unwrap_or("");
    match key.id {
        "sigma" => { sigma(p,t,key.cx-2.1,cy,6.5,color); bold_txt(p,t,key.cx+3.9,cy,6.7,color,"−"); }
        "indirect" => swap(p,t,key.cx,cy,"x","I",6.7,color),
        "7" => swap(p,t,key.cx,cy,"x","y",6.9,color),
        "8" => r_arrow(p,t,key.cx,cy,false,6.8,color),
        "9" => r_arrow(p,t,key.cx,cy,true,6.8,color),
        "4" => one_over_x(p,t,key.cx,cy,6.8,color),
        "5" => power(p,t,key.cx,cy,"y","x",6.8,color,color),
        "2" => pi(p,t,key.cx,cy,7.0,color),
        "enter" => {
            let x = match key.sub_align { SubAlign::Right => key.cx+30.0, SubAlign::Center => key.cx };
            bold_txt_aligned(p,t,x,cy,6.7,color,"DEG",if matches!(key.sub_align,SubAlign::Right){Align2::RIGHT_CENTER}else{Align2::CENTER_CENTER});
        }
        _ => { let size=if key.w>=70.0{6.7}else if sub.len()>=5{6.0}else{6.6}; bold_txt(p,t,key.cx,cy,size,color,sub); }
    }
}

fn draw_legends(p:&Painter,t:Transform) {
    one_over_x(p,t,65.0,151.8,10.5,WHITE);
    sqrt_x(p,t,116.0,151.7,10.6,WHITE);
    power(p,t,167.0,151.7,"y","x",10.4,WHITE,WHITE);
    r_arrow(p,t,218.0,151.7,false,10.2,WHITE);
    swap(p,t,269.0,151.7,"x","y",10.0,WHITE);
    for (x,s) in [(65.0,"a"),(116.0,"b"),(167.0,"c"),(218.0,"d"),(269.0,"e")] { bold_txt(p,t,x,207.1,7.9,YELLOW,s); }
    xbar(p,t,55.0,263.0,7.4,YELLOW); bold_txt(p,t,71.7,263.0,7.8,CYAN,"s");
    bold_txt(p,t,107.5,263.0,7.8,YELLOW,"GSB"); bold_txt(p,t,126.2,263.0,7.8,CYAN,"f");
    bold_txt(p,t,156.0,263.0,7.8,YELLOW,"FIX"); bold_txt(p,t,179.0,263.0,7.8,CYAN,"SCI");
    bold_txt(p,t,218.0,263.0,7.8,YELLOW,"RND"); bold_txt(p,t,258.5,263.0,7.8,YELLOW,"LBL"); bold_txt(p,t,280.0,263.0,7.8,CYAN,"f");
    bold_txt(p,t,158.0,316.6,7.8,YELLOW,"DSZ"); bold_txt(p,t,180.7,316.6,7.8,CYAN,"(i)");
    bold_txt(p,t,209.5,316.6,7.8,YELLOW,"ISZ"); bold_txt(p,t,232.0,316.6,7.8,CYAN,"(i)");
    bold_txt(p,t,61.0,374.0,7.9,YELLOW,"W/DATA"); bold_txt(p,t,116.0,374.0,7.9,CYAN,"MERGE");
    swap_two_color(p,t,167.0,374.0,"P","S",7.7,YELLOW,YELLOW);
    bold_txt(p,t,218.0,374.0,7.9,YELLOW,"CL REG"); bold_txt(p,t,269.0,374.0,7.9,YELLOW,"CL PRGM");
    eq_pair(p,t,60.5,424.6,false);
    bold_txt(p,t,108.0,424.6,8.0,YELLOW,"LN"); power(p,t,128.0,424.6,"e","x",7.9,CYAN,CYAN);
    bold_txt(p,t,181.0,424.6,8.0,YELLOW,"LOG"); power(p,t,208.0,424.6,"10","x",7.9,CYAN,CYAN);
    sqrt_x(p,t,252.0,424.6,8.0,YELLOW); power(p,t,279.0,424.6,"x","2",7.9,CYAN,CYAN);
    eq_pair(p,t,60.5,476.8,true);
    inverse_trig(p,t,116.0,476.8,"SIN"); inverse_trig(p,t,190.5,476.8,"COS"); inverse_trig(p,t,263.5,476.8,"TAN");
    relation_pair(p,t,60.5,529.0,'<',true);
    swap_two_color(p,t,117.5,529.0,"R","P",7.7,YELLOW,CYAN);
    swap_two_color(p,t,190.5,529.0,"D","R",7.7,YELLOW,CYAN);
    swap_two_color(p,t,263.5,529.0,"H","H.MS",7.3,YELLOW,CYAN);
    relation_pair(p,t,60.5,581.0,'>',false);
    bold_txt(p,t,106.0,581.0,7.9,YELLOW,"%"); bold_txt(p,t,126.5,581.0,7.9,CYAN,"%CH");
    bold_txt(p,t,181.0,581.0,7.9,YELLOW,"INT"); bold_txt(p,t,207.0,581.0,7.9,CYAN,"FRAC");
    bold_txt(p,t,253.0,581.0,7.9,YELLOW,"−x−"); bold_txt(p,t,281.0,581.0,7.9,CYAN,"STK");
}

fn palette(style:KeyStyle)->(Color32,Color32,Color32,Color32,Color32){
    match style {
        KeyStyle::Olive=>(Color32::from_rgb(183,181,103),Color32::from_rgb(133,126,61),Color32::from_rgb(102,95,48),WHITE,DARK),
        KeyStyle::Orange=>(Color32::from_rgb(246,181,43),Color32::from_rgb(197,126,18),Color32::from_rgb(145,84,7),DARK,DARK),
        KeyStyle::Blue=>(Color32::from_rgb(69,194,220),Color32::from_rgb(31,137,162),Color32::from_rgb(18,92,109),DARK,DARK),
        KeyStyle::White=>(Color32::from_rgb(236,237,232),Color32::from_rgb(192,196,193),Color32::from_rgb(148,153,151),DARK,DARK),
        KeyStyle::Black=>(Color32::from_rgb(22,24,25),Color32::from_rgb(10,11,12),Color32::from_rgb(4,5,5),WHITE,WHITE),
    }
}

fn bold_txt(p:&Painter,t:Transform,x:f32,y:f32,size:f32,color:Color32,s:&str){ bold_txt_aligned(p,t,x,y,size,color,s,Align2::CENTER_CENTER) }
fn bold_txt_aligned(p:&Painter,t:Transform,x:f32,y:f32,size:f32,color:Color32,s:&str,align:Align2){
    let font=FontId::new(t.s(size),FontFamily::Proportional); let pos=t.pos(x,y);
    p.text(pos,align,s,font.clone(),color); p.text(Pos2::new(pos.x+t.s(0.28),pos.y),align,s,font,color);
}
fn darker(c:Color32,n:u8)->Color32{Color32::from_rgb(c.r().saturating_sub(n),c.g().saturating_sub(n),c.b().saturating_sub(n))}
fn cross(p:&Painter,t:Transform,cx:f32,cy:f32,h:f32,c:Color32){p.line_segment([t.pos(cx-h,cy-h),t.pos(cx+h,cy+h)],Stroke::new(t.s(1.05),c));p.line_segment([t.pos(cx-h,cy+h),t.pos(cx+h,cy-h)],Stroke::new(t.s(1.05),c));}
fn divide(p:&Painter,t:Transform,cx:f32,cy:f32,h:f32,c:Color32){p.line_segment([t.pos(cx-h,cy),t.pos(cx+h,cy)],Stroke::new(t.s(1.05),c));p.circle_filled(t.pos(cx,cy-4.1),t.s(0.9),c);p.circle_filled(t.pos(cx,cy+4.1),t.s(0.9),c);}
fn triangle(p:&Painter,t:Transform,cx:f32,cy:f32,h:f32,up:bool,c:Color32){let d=if up{-1.0}else{1.0};p.add(Shape::convex_polygon(vec![t.pos(cx,cy+d*h),t.pos(cx-h,cy-d*h*0.72),t.pos(cx+h,cy-d*h*0.72)],c,Stroke::NONE));}
fn sigma(p:&Painter,t:Transform,cx:f32,cy:f32,size:f32,c:Color32){let s=size/8.0;let l=cx-4.0*s;let r=cx+4.0*s;let top=cy-4.0*s;let bot=cy+4.0*s;let st=Stroke::new(t.s(0.95*s),c);p.line_segment([t.pos(l,top),t.pos(r,top)],st);p.line_segment([t.pos(l,top),t.pos(cx+0.8*s,cy)],st);p.line_segment([t.pos(cx+0.8*s,cy),t.pos(l,bot)],st);p.line_segment([t.pos(l,bot),t.pos(r,bot)],st);}
fn pi(p:&Painter,t:Transform,cx:f32,cy:f32,size:f32,c:Color32){let s=size/7.0;let st=Stroke::new(t.s(0.8*s),c);p.line_segment([t.pos(cx-4.0*s,cy-2.7*s),t.pos(cx+4.0*s,cy-2.7*s)],st);p.line_segment([t.pos(cx-2.1*s,cy-2.7*s),t.pos(cx-2.1*s,cy+2.8*s)],st);p.line_segment([t.pos(cx+2.1*s,cy-2.7*s),t.pos(cx+2.1*s,cy+2.8*s)],st);}
fn one_over_x(p:&Painter,t:Transform,cx:f32,cy:f32,size:f32,c:Color32){let s=size/9.0;bold_txt(p,t,cx-6.0*s,cy,size,c,"1");p.line_segment([t.pos(cx-1.6*s,cy+4.0*s),t.pos(cx+2.3*s,cy-4.0*s)],Stroke::new(t.s(0.85*s),c));bold_txt(p,t,cx+6.0*s,cy,size,c,"x");}
fn sqrt_x(p:&Painter,t:Transform,cx:f32,cy:f32,size:f32,c:Color32){let s=size/9.0;let x0=cx-8.2*s;let st=Stroke::new(t.s(1.0*s),c);p.line_segment([t.pos(x0,cy+0.3*s),t.pos(x0+2.5*s,cy+4.0*s)],st);p.line_segment([t.pos(x0+2.5*s,cy+4.0*s),t.pos(x0+5.8*s,cy-5.0*s)],st);p.line_segment([t.pos(x0+5.8*s,cy-5.0*s),t.pos(x0+15.0*s,cy-5.0*s)],Stroke::new(t.s(0.8*s),c));bold_txt(p,t,cx+3.5*s,cy+0.4*s,size,c,"x");}
fn power(p:&Painter,t:Transform,cx:f32,cy:f32,base:&str,exp:&str,size:f32,bc:Color32,ec:Color32){let bw=if base.len()>1{5.5}else{2.6};bold_txt(p,t,cx-bw,cy+1.0,size,bc,base);bold_txt(p,t,cx+bw+2.4,cy-size*0.42,size*0.65,ec,exp);}
fn r_arrow(p:&Painter,t:Transform,cx:f32,cy:f32,up:bool,size:f32,c:Color32){bold_txt(p,t,cx-3.2,cy,size,c,"R");triangle(p,t,cx+5.8,cy+0.2,size*0.3,up,c);}
fn swap(p:&Painter,t:Transform,cx:f32,cy:f32,l:&str,r:&str,size:f32,c:Color32){swap_two_color(p,t,cx,cy,l,r,size,c,c)}
fn swap_two_color(p:&Painter,t:Transform,cx:f32,cy:f32,l:&str,r:&str,size:f32,lc:Color32,rc:Color32){let span=if r.len()>1{11.0}else{8.0};bold_txt(p,t,cx-span,cy,size,lc,l);bold_txt(p,t,cx+span,cy,size,rc,r);double_arrow(p,t,cx,cy,size*0.52,if lc==rc{lc}else{CYAN});}
fn double_arrow(p:&Painter,t:Transform,cx:f32,cy:f32,h:f32,c:Color32){let x1=cx-h;let x2=cx+h;let yu=cy-1.35;let yd=cy+1.35;let st=Stroke::new(t.s(0.7),c);p.line_segment([t.pos(x1,yu),t.pos(x2,yu)],st);p.line_segment([t.pos(x2-2.0,yu-1.4),t.pos(x2,yu)],st);p.line_segment([t.pos(x2-2.0,yu+1.4),t.pos(x2,yu)],st);p.line_segment([t.pos(x2,yd),t.pos(x1,yd)],st);p.line_segment([t.pos(x1+2.0,yd-1.4),t.pos(x1,yd)],st);p.line_segment([t.pos(x1+2.0,yd+1.4),t.pos(x1,yd)],st);}
fn xbar(p:&Painter,t:Transform,cx:f32,cy:f32,size:f32,c:Color32){bold_txt(p,t,cx,cy+0.4,size,c,"x");p.line_segment([t.pos(cx-3.4,cy-4.2),t.pos(cx+3.4,cy-4.2)],Stroke::new(t.s(0.8),c));}
fn inverse_trig(p:&Painter,t:Transform,cx:f32,cy:f32,name:&str){bold_txt(p,t,cx-2.0,cy,7.9,YELLOW,name);bold_txt(p,t,cx+13.0,cy-4.1,5.2,CYAN,"−1");}
fn eq_pair(p:&Painter,t:Transform,cx:f32,cy:f32,ne:bool){bold_txt(p,t,cx-13.0,cy,7.5,YELLOW,"x");if ne{not_eq(p,t,cx-5.0,cy,YELLOW)}else{bold_txt(p,t,cx-5.0,cy,7.5,YELLOW,"=")};bold_txt(p,t,cx+1.0,cy,7.5,YELLOW,"0");bold_txt(p,t,cx+7.0,cy,7.5,CYAN,"x");if ne{not_eq(p,t,cx+15.0,cy,CYAN)}else{bold_txt(p,t,cx+15.0,cy,7.5,CYAN,"=")};bold_txt(p,t,cx+22.0,cy,7.5,CYAN,"y");}
fn not_eq(p:&Painter,t:Transform,cx:f32,cy:f32,c:Color32){let st=Stroke::new(t.s(0.75),c);p.line_segment([t.pos(cx-2.8,cy-1.5),t.pos(cx+2.8,cy-1.5)],st);p.line_segment([t.pos(cx-2.8,cy+1.5),t.pos(cx+2.8,cy+1.5)],st);p.line_segment([t.pos(cx-2.3,cy+3.2),t.pos(cx+2.3,cy-3.2)],st);}
fn relation_pair(p:&Painter,t:Transform,cx:f32,cy:f32,op:char,incl:bool){let os=if op=='<'{"<"}else{">"};bold_txt(p,t,cx-13.0,cy,7.4,YELLOW,"x");bold_txt(p,t,cx-6.4,cy,7.4,YELLOW,os);bold_txt(p,t,cx,cy,7.4,YELLOW,"0");bold_txt(p,t,cx+7.0,cy,7.4,CYAN,"x");relop(p,t,cx+15.0,cy,op,incl,CYAN);bold_txt(p,t,cx+23.0,cy,7.4,CYAN,"y");}
fn relop(p:&Painter,t:Transform,cx:f32,cy:f32,op:char,incl:bool,c:Color32){let f=if op=='<'{1.0}else{-1.0};let st=Stroke::new(t.s(0.75),c);p.line_segment([t.pos(cx+2.5*f,cy-3.0),t.pos(cx-2.0*f,cy)],st);p.line_segment([t.pos(cx-2.0*f,cy),t.pos(cx+2.5*f,cy+3.0)],st);if incl{p.line_segment([t.pos(cx-2.5,cy+4.3),t.pos(cx+2.8,cy+4.3)],st);}}
