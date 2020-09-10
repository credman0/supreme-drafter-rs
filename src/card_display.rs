 
use yew::prelude::*;

#[derive(Clone)]
pub struct CardDisplay {
    link: ComponentLink<Self>,
    pub name: String,
    url: String,
    selected: bool,
    class: String,
    onsignal: Callback<CardDisplay>,
}

impl std::cmp::PartialEq<CardDisplay> for CardDisplay {
    fn eq (&self, other:&CardDisplay) -> bool {
        return self.name == *other.name;
    }
}

#[derive(Clone, PartialEq, Properties)]
pub struct Props {
    #[prop_or_default]
    pub name: String,
    pub url: String,
    pub selected: bool,
    pub onsignal: Callback<CardDisplay>,
}

pub enum Msg {
    Clicked(),
}

fn get_class_string (additional:String) -> String {
    return format!("card-img-top card-block d-flex card-img {}", additional)
}

impl CardDisplay {
    fn set_selected (&mut self, is_selected:bool) {
        if self.selected != is_selected {
            self.selected = is_selected;
            if self.selected {
                self.class = get_class_string(String::from("selected border border-primary"));
            } else {
                self.class = get_class_string(String::from(""));
            }
        }
    }

    pub fn get_selected (&self) -> bool {
        return self.selected;
    }
}

impl Component for CardDisplay {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let mut this = CardDisplay {
            link,
            name: props.name,
            url: props.url,
            selected: props.selected,
            class: get_class_string(String::new()),
            onsignal: props.onsignal,
        };
        this.set_selected(this.selected);
        return this
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Clicked() => {
                // self.set_selected(!self.selected);
                self.onsignal.emit(self.clone());
            }
        }
        false
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.name = props.name;
        self.url = props.url;
        self.set_selected(props.selected);
        self.onsignal = props.onsignal;
        true
    }

    fn view(&self) -> Html {
        html! {
            <div class="card shadow-sm mx-1 px-1 pt-1 pb-1 mt-1 ">
                <img class=&self.class src=self.url alt=self.name onclick=self.link.callback(|_| Msg::Clicked())/>
                <div class="card-body align-items-center d-flex justify-content-center">
                    <p class="card-text"><b>{&self.name}</b></p>
                </div>
            </div>
        }
    }
}