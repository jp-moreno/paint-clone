use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::HtmlElement;
use web_sys::{console, window, ClipboardItem, CanvasRenderingContext2d, HtmlCanvasElement, HtmlInputElement, MouseEvent};
use yew::prelude::*;

trait AbstractShape {
    fn draw(&self, canvas: &CanvasRenderingContext2d);
}

struct Rect {
    x1: f64,
    y1: f64,
    x2: f64,
    y2: f64,
    color: Color,
}


impl AbstractShape for Rect{
    fn draw(&self, canvas_context: &CanvasRenderingContext2d) {
        draw_rect(canvas_context, self.x1, self.y1, self.x2, self.y2, self.color);
    }
}


struct Circle {
    x: f64,
    y: f64,
    r: f64,
    color: Color,
}

impl AbstractShape for Circle{
    fn draw(&self, canvas_context: &CanvasRenderingContext2d) {
        draw_at_position(canvas_context, self.x, self.y, self.color);
    }
}



#[derive(Debug, Clone, Copy)]
struct Color {
    r: u8,
    g: u8,
    b: u8,
    a: Option<f64>,
}


impl Color {
    fn to_rgb_str(self) -> String{
        match self.a {
            Some(alpha) => format!("rgba({}, {}, {}, {})", self.r, self.g, self.b, alpha),
            None => format!("rgb({}, {}, {})", self.r, self.g, self.b)
        }
    }

   fn from_hex_str(hex: &str) -> Result<Color, &'static str> {
        // Remove # if present
        let hex = hex.trim_start_matches('#');
        
        // Validate hex string length
        if hex.len() != 6  && hex.len() != 8{
            return Err("Invalid hex color format");
        }
        
        // Parse each color component
        let r = u8::from_str_radix(&hex[0..2], 16)
            .map_err(|_| "Invalid red component")?;
        let g = u8::from_str_radix(&hex[2..4], 16)
            .map_err(|_| "Invalid green component")?;
        let b = u8::from_str_radix(&hex[4..6], 16)
            .map_err(|_| "Invalid blue component")?;

        let mut alpha = None;
        if hex.len() == 8 {
            alpha = u8::from_str_radix(&hex[6..8], 16).ok();
        } 

        let a = alpha.map(|x| f64::from(x) /255.0);

        
        Ok(Color { r, g, b, a})
    }
}


trait DrawingTool {
    fn draw(&mut self, canvas_context: &CanvasRenderingContext2d, tooltip_canvas: &CanvasRenderingContext2d, drawn_objects: &mut Vec<Box<dyn AbstractShape>>, event: &MouseEvent);
    fn start_draw(&mut self, canvas_context: &CanvasRenderingContext2d, tooltip_canvas: &CanvasRenderingContext2d, drawn_objects: &mut Vec<Box<dyn AbstractShape>>, event: &MouseEvent);
    fn end_draw(&mut self, canvas_context: &CanvasRenderingContext2d, tooltip_canvas: &CanvasRenderingContext2d, drawn_objects: &mut Vec<Box<dyn AbstractShape>>, event: &MouseEvent);
    fn change_primary_color(&mut self, color: Color);
    fn change_secondary_color (&mut self, color: Color);
}


struct BrushTool {
    size: f64,
    color: Color,
}


impl DrawingTool for BrushTool {
    fn draw(&mut self, canvas_context: &CanvasRenderingContext2d, _tooltip_canvas: &CanvasRenderingContext2d, drawn_objects: &mut Vec<Box<dyn AbstractShape>>, event: &MouseEvent){
        let circle = Circle {x: event.offset_x() as f64, y: event.offset_y() as f64, r: self.size, color: self.color};
        drawn_objects.push(Box::new(circle));
    }

    fn start_draw(&mut self, canvas_context: &CanvasRenderingContext2d, tooltip_canvas: &CanvasRenderingContext2d, drawn_objects: &mut Vec<Box<dyn AbstractShape>>, event: &MouseEvent) {
        self.draw(canvas_context, tooltip_canvas, drawn_objects, event);
    }

    fn change_primary_color(&mut self, color: Color) {
        self.color = color;
    }

    fn end_draw(&mut self, _canvas_context: &CanvasRenderingContext2d, _tooltip_canvas: &CanvasRenderingContext2d, _drawn_objects: &mut Vec<Box<dyn AbstractShape>>, _event: &MouseEvent) {}
    fn change_secondary_color (&mut self, _color: Color) {}
}

struct RectTool {
    x: f64,
    y: f64,
    color: Color,
    tooltip_color: Color,
}


impl DrawingTool for RectTool {
    fn draw(&mut self, canvas_context: &CanvasRenderingContext2d, tooltip_canvas: &CanvasRenderingContext2d, drawn_objects: &mut Vec<Box<dyn AbstractShape>>, event: &MouseEvent){
        clear_canvas(tooltip_canvas, 500.0, 500.0);
        draw_rect(tooltip_canvas, self.x, self.y, event.offset_x() as f64, event.offset_y() as f64, self.tooltip_color);
    }

    fn start_draw(&mut self, canvas_context: &CanvasRenderingContext2d, tooltip_canvas: &CanvasRenderingContext2d, drawn_objects: &mut Vec<Box<dyn AbstractShape>>, event: &MouseEvent) {
        self.x = event.offset_x() as f64;
        self.y = event.offset_y() as f64;
    }

    fn end_draw(&mut self, canvas_context: &CanvasRenderingContext2d, tooltip_canvas: &CanvasRenderingContext2d, drawn_objects: &mut Vec<Box<dyn AbstractShape>>, event: &MouseEvent) {

        clear_canvas(tooltip_canvas, 500.0, 500.0);
        let rect = Rect {x1: self.x, y1: self.y, x2: event.offset_x() as f64, y2: event.offset_y() as f64, color: self.color};
        drawn_objects.push(Box::new(rect));

    }


    fn change_primary_color(&mut self, color: Color) {
        self.color = color;
    }

    fn change_secondary_color (&mut self, _color: Color) {}
}


// Canvas state management
pub struct CanvasState {
    current_tool: Box<dyn DrawingTool>,
    mouse_pressed: bool,
    primary_color: Color,
    secondary_color: Color,
    tooltip_color: Color,
    drawn_objects: Vec<Box<dyn AbstractShape>>,
    undo_stack: Vec<Box<dyn AbstractShape>>,
}

impl CanvasState {
    fn new() -> Self {
        Self {
            current_tool: Box::new(BrushTool { 
                size: 5.0,
                color: Color{r: 0, g: 0, b: 255, a: None},
            }),
            mouse_pressed: false,
            primary_color: Color{r: 0, g: 0, b: 255, a: None},
            secondary_color: Color{r: 255, g: 255, b: 255, a: None},
            tooltip_color: Color{r: 200, g: 0, b: 255, a: Some(0.5)},
            drawn_objects: vec![],
            undo_stack: vec![],
        }
    }
}


// Yew component with improved structure
pub struct CanvasComponent {
    canvas_ref: NodeRef,
    tool_canvas_ref: NodeRef,
    state: CanvasState,
    height: u32,
    width: u32,
}

pub enum Msg {
    MouseDown(MouseEvent),
    MouseUp(MouseEvent),
    MouseMove(MouseEvent),
    ChangeTool(Box<dyn DrawingTool>),
    SaveImage,
    ClearCanvas,
    ChangeColor(Event),
    SelectRectTool,
    SelectBrushTool,
    Undo,
    Redo,
}

impl Component for CanvasComponent {
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            canvas_ref: NodeRef::default(),
            tool_canvas_ref: NodeRef::default(),
            state: CanvasState::new(),
            height: 500,
            width: 500,
        }
    }


    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        if let Some(canvas) = self.canvas_ref.cast::<HtmlCanvasElement>() {
            let canvas_context = canvas
                .get_context("2d")
                .unwrap()
                .unwrap()
                .dyn_into::<CanvasRenderingContext2d>()
                .unwrap();

            if let Some(tool_canvas) = self.tool_canvas_ref.cast::<HtmlCanvasElement>() {
                let tool_context = tool_canvas
                    .get_context("2d")
                    .unwrap()
                    .unwrap()
                    .dyn_into::<CanvasRenderingContext2d>()
                    .unwrap();

                match msg {
                    Msg::MouseDown(event) => {
                        self.state.mouse_pressed = true;
                        self.state.current_tool.start_draw(&canvas_context, &tool_context, &mut self.state.drawn_objects, &event);
                    }
                    Msg::MouseMove(event) if self.state.mouse_pressed => {
                        self.state.current_tool.draw(&canvas_context, &tool_context, &mut self.state.drawn_objects, &event);
                    }
                    Msg::MouseUp(event) => {
                        self.state.mouse_pressed = false;
                        self.state.current_tool.end_draw(&canvas_context, &tool_context, &mut self.state.drawn_objects, &event);
                    }
                    Msg::ChangeTool(tool) => {
                        self.state.current_tool = tool;
                    }
                    Msg::SaveImage => {
                        let _ = canvas.to_data_url().map(|data_url| {
                            let link = window().unwrap().document().unwrap().create_element("a").unwrap();
                            link.set_attribute("href", &data_url).unwrap();
                            link.set_attribute("download", "scribblai.png").unwrap(); // Sets the download attribute
                            link.set_attribute("style", "display: none").unwrap(); // Hide the link element

                            let body = window().unwrap().document().unwrap().body().unwrap();
                            body.append_child(&link).unwrap();
                            if let Ok(html_element) = link.dyn_into::<HtmlElement>() { 
                                html_element.click();
                                body.remove_child(&html_element).unwrap(); // Clean up
                            }
                        });
                    }
                    Msg::ClearCanvas => {
                        self.state.drawn_objects.clear();
                        self.state.undo_stack.clear();

                    }
                    Msg::ChangeColor(event) => {
                        if let Some(input) = event.target_dyn_into::<HtmlInputElement>() {
                            let color = Color::from_hex_str(&input.value()).expect("ERROR CONVERTING COLOR");
                            self.state.primary_color = color;
                            self.state.current_tool.change_primary_color(color);
                        }
                        return false;
                    }
                    Msg::SelectRectTool => {
                        self.state.current_tool = Box::new(RectTool{ x: 0.0, y:0.0, color: self.state.primary_color, tooltip_color:self.state.tooltip_color});
                        return false
                    }
                    Msg::SelectBrushTool => {
                        self.state.current_tool = Box::new(BrushTool{size:0.0, color: self.state.primary_color});
                        return false;
                    }
                    Msg::Undo => {
                        if let Some(obj) = self.state.drawn_objects.pop() {
                            self.state.undo_stack.push(obj);
                        }
                    }
                    Msg::Redo => {
                        if let Some(obj) = self.state.undo_stack.pop() {
                            self.state.drawn_objects.push(obj);
                        }
                    }
                    _ => {
                        console::log_1(&"not implemented".into());
                        return false;
                    }
                }


                let color = Color {r: 255, g: 255, b: 255, a: None};
                draw_rect(&canvas_context, 0.0, 0.0, self.width as f64, self.height as f64, color);
                for shape in &self.state.drawn_objects {
                    shape.draw(&canvas_context);
                }
                return true;

            } else {
                console::log_1(&"Tool canvas reference not ready".into());
            }



        } else {
            console::log_1(&"Canvas reference is not ready yet.".into());
        }
        false
}


    fn rendered(&mut self, _ctx: &Context<Self>, _first_render: bool) {
        if let Some(canvas) = self.canvas_ref.cast::<HtmlCanvasElement>() {
            if _first_render {
                canvas.set_width(self.width);
                canvas.set_height(self.height);
            }
            let context = canvas
                .get_context("2d")
                .unwrap()
                .unwrap()
                .dyn_into::<CanvasRenderingContext2d>()
                .unwrap();
            if _first_render{
                    let color = Color {r: 255, g: 255, b: 255, a: None};
                    draw_rect(&context, 0.0, 0.0, self.width as f64, self.height as f64, color);
            }
        }

        if let Some(canvas) = self.tool_canvas_ref.cast::<HtmlCanvasElement>() {
            if _first_render {
                canvas.set_width(self.width);
                canvas.set_height(self.height);
            }
        }

    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let onmousedown = ctx.link().callback(Msg::MouseDown);
        let onmouseup = ctx.link().callback(Msg::MouseUp);
        let onmousemove = ctx.link().callback(Msg::MouseMove);
        let onsaveimage = ctx.link().callback(|_| Msg::SaveImage);
        let onclear = ctx.link().callback(|_| Msg::ClearCanvas);
        let changecolor = ctx.link().callback(Msg::ChangeColor);
        let select_brushtool = ctx.link().callback(|_| Msg::SelectBrushTool);
        let select_recttool = ctx.link().callback(|_| Msg::SelectRectTool);
        let undo = ctx.link().callback(|_| Msg::Undo);
        let redo = ctx.link().callback(|_| Msg::Redo);

        html! {
            <div>
                <canvas 
                    ref={self.tool_canvas_ref.clone()}
                    style="border:1px solid transparent; position: absolute; top: 0; left: 0; z-index: 2; pointer-events: none;"
                />
                <canvas
                    ref={self.canvas_ref.clone()}
                    style="border:1px solid black; position: absolute; top: 0; left: 0; z-index: 1;"
                    {onmousedown}
                    {onmouseup}
                    {onmousemove}
                />
                <div id="toolbar">
                    <button onclick={select_brushtool}>{"Brush Tool"}</button>
                    <button onclick={select_recttool}>{"Rect Tool"}</button>
                    <button onclick={onclear}>{"Clear"}</button>
                    <button onclick={onsaveimage}>{"Save"}</button>
                    <button onclick={undo}>{"Undo"}</button>
                    <button onclick={redo}>{"Redo"}</button>
                    <input type="color" onchange={changecolor} />
                </div>
            </div>
        }
    }
}

/// Draws a small circle at the given position on the canvas
fn draw_at_position(context: &CanvasRenderingContext2d, x: f64, y: f64, color: Color) {
    context.begin_path();
    context.arc(x, y, 5.0, 1.0, std::f64::consts::PI * 2.0).unwrap();
    context.set_fill_style_str(&Color::to_rgb_str(color));
    context.fill();
}


fn draw_rect(context: &CanvasRenderingContext2d, x1: f64, y1: f64, x2: f64, y2: f64, color: Color) {
    context.set_fill_style_str(&Color::to_rgb_str(color));
    context.fill_rect(x1, y1, x2-x1, y2-y1);
}

fn clear_canvas(context: &CanvasRenderingContext2d, canvas_width: f64, canvas_height: f64){
    context.clear_rect(0.0, 0.0, canvas_width, canvas_height);
}
