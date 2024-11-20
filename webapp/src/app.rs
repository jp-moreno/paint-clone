use yew::prelude::*;
use crate::components::paint::CanvasComponent;


#[function_component(App)]
pub fn app() -> Html {
    html! {
        <main>
            <CanvasComponent />
        </main>
    }
}
