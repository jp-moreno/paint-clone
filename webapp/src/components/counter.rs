use yew::prelude::*;

pub struct Counter {
    counter: i32,
}

pub enum Msg {
    Increment,
}

impl Component for Counter {
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Counter {counter: 0}
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Increment => {
                self.counter += 1;
                true
            }
        }
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        html! {
            <div>
                <p>{ format!("Current count: {}", self.counter) }</p>
                <button onclick={_ctx.link().callback(|_| Msg::Increment)}>{ "Increment" }</button>
            </div>
        }
    }
}
