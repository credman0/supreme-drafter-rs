#![recursion_limit="2048"]
mod utils;
mod card_display;

use wasm_bindgen::prelude::*;
use std::collections::HashMap;

use card_display::CardDisplay;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

static SET_JSON_STR:&'static str = include_str!("small-sets.json");

const NUM_PACKS:i16 = 18;
const PICKS_PER_PACK:i16 = 2;

use yew::prelude::*;

use serde_json::{Value, Map};

struct Model {
    link: ComponentLink<Self>,
    set_generator:SetGenerator,
    in_draft:bool,
    setup_info:SetupInfo,
    pack:Pack,
    sorted_picks: Vec<Vec<Card>>,
    selected: Vec<CardDisplay>,
}
struct SetupInfo {
    sets:Vec<(String, i16)>
}
struct Pack {
    cards:Vec<Card>,
    num_picks:i16
}

#[derive(Clone)]
struct Card {
    cmc:i16,
    name:String,
    img_url:String,
    selected:bool
}

impl std::cmp::PartialEq<Card> for Card {
    fn eq (&self, other:&Card) -> bool {
        return self.name == *other.name;
    }
}

enum Msg {
    Select(CardDisplay),
    Confirm(),
    Export(),
    StartDraft(),
    AddSet(),
    RemoveSet(String),
    ChangeSet(String,String),
    ChangeSetNumCards(String, i16),
    DoNothing()
}

#[derive(Clone)]
struct SetGenerator {
    all_set_json:Map<String, Value>,
    mythics:Vec<Card>,
    rares:Vec<Card>,
    uncommons:Vec<Card>,
    commons:Vec<Card>,
    basics:Vec<Card>,
    pack_number:i16,
    pack_series:Vec<String>
}

pub fn shuffle<T>(vec:&mut Vec<T>) {
    for i in 0..vec.len() {
        let j = (js_sys::Math::random() * (vec.len() as f64)) as usize;
        vec.swap(i, j);
    }
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    pub fn log(s: &str);
}

impl SetGenerator {
    // pub fn new (all_set_json:Value, init_set:&str) -> Self {
    //     let mut generator = SetGenerator{all_set_json:all_set_json.as_object().unwrap().clone(), mythics:vec![], rares:vec![], uncommons:vec![], commons:vec![], basics:vec![], pack_number:0, pack_series:vec![]};
    //     generator.prepare_set(init_set);
    //     generator
    // }

    pub fn new (all_set_json:Value) -> Self {
        let generator = SetGenerator{all_set_json:all_set_json.as_object().unwrap().clone(), mythics:vec![], rares:vec![], uncommons:vec![], commons:vec![], basics:vec![], pack_number:0, pack_series:vec![]};
        generator
    }

    fn is_basic(&self, card_name:&str) -> bool {
        match card_name {
            "Forest" | "Mountain" | "Swamp" | "Plains" | "Island" => return true, 
            "Snow-Covered Forest" | "Snow-Covered Mountain" | "Snow-Covered Swamp" | "Snow-Covered Plains" | "Snow-Covered Island" => return true, 
            _ => return false
        }
    }
    pub fn init_from_setup_packs (&mut self, setup_list:&Vec<(String, i16)>, num_random_packs:i16) {
        let mut set_list = vec![];
        // track one of each set for usage in making the random sets
        let mut all_set_selection_list = vec![];
        for setup in setup_list {
            all_set_selection_list.push(setup.0.clone());
            for _ in 0..setup.1 {
                set_list.push(setup.0.clone());
            }
        }
        // This insertion could of course be a binary search instead but I don't want to do that
        for _ in 0..num_random_packs { 
            let idx = (js_sys::Math::random() * (all_set_selection_list.len() as f64)) as usize;
            set_list.push(all_set_selection_list[idx].clone());
        }
        shuffle(&mut set_list);
        self.pack_series = set_list;
        let init_set = self.pack_series[0].clone();
        self.prepare_set(init_set.as_str());
    }
    fn prepare_set(&mut self, set_name:&str) {
        log(format!("preparing set {}", set_name).as_str());
        let set_json = self.all_set_json[set_name].as_object().unwrap();
        self.rares = vec![];
        self.uncommons = vec![];
        self.commons = vec![];
        self.basics = vec![];
        for card_name in set_json.keys() {
            let value = set_json[card_name].as_object().unwrap();
            let cmc = value["c"].as_i64().unwrap();
            let rarity = value["r"].as_str().unwrap();
            let id = value["i"].as_i64().unwrap();
            let url = get_img_url(id);
            let card :Card= Card{name:String::from(card_name), cmc:cmc as i16, img_url:url, selected:false};
            if self.is_basic(card_name) {
                self.basics.push(card)
            } else {
                match rarity {
                    "r" => self.rares.push(card),
                    "m" => self.mythics.push(card),
                    "u" => self.uncommons.push(card),
                    "c" => self.commons.push(card),
                    _ => {} 
                }
            }
        }
        log(format!("{}-m, {}-r, {}-u, {}-c, {}-b", self.mythics.len(), self.rares.len(), self.uncommons.len(), self.commons.len(), self.basics.len()).as_str());
    }

    fn pull_card_from_pool(&self, pool:&Vec<Card>) -> Card {
        let idx = (js_sys::Math::random() * pool.len() as f64) as usize;
        return pool[idx].clone();
    }

    fn progress_pack(&mut self) {
        let current_set = self.pack_series[self.pack_number as usize].clone();
        self.pack_number += 1;
        if self.pack_number < self.pack_series.len() as i16 {
            let next_set = self.pack_series[self.pack_number as usize].clone();
            if current_set!=next_set {
                self.prepare_set(next_set.as_str());
            }
        }
    }
    fn generate_card(&self, rarity:char) -> Card {
        match rarity {
            'r' => {
                if self.mythics.len()==0 {
                    return self.pull_card_from_pool(&self.rares)
                }
                if js_sys::Math::random() < (1./8.){
                    // mythics 1 in 8 packs
                    return self.pull_card_from_pool(&self.mythics)
                } else {
                    return self.pull_card_from_pool(&self.rares)
                }
            },
            'u' => self.pull_card_from_pool(&self.uncommons),
            'c' => self.pull_card_from_pool(&self.commons),
            'b' => self.pull_card_from_pool(&self.basics),
            _ => {Card{cmc:1, name:String::from("foo"), img_url:String::from("https://gatherer.wizards.com/Handlers/Image.ashx?multiverseid=235596&type=card"), selected:false}} // fake card
        }
    }
    fn generate_pack(&self) -> Pack {
        let mut cards = vec![self.generate_card('r')];
        for _ in 0..3 {
            let mut card = self.generate_card('u');
            while cards.contains(&card) {
                card = self.generate_card('u');
            }
            cards.push(card.clone());
        }
        for _ in 0..10 {
            let mut card = self.generate_card('c');
            while cards.contains(&card) {
                card = self.generate_card('c');
            }
            cards.push(card.clone());
        }
        if self.basics.len()>0 {
            cards.push(self.generate_card('b'));
        }
        Pack{cards:cards, num_picks:0}
    }
}

pub fn get_img_url (multiverse_id:i64) -> String {
    return format!("https://gatherer.wizards.com/Handlers/Image.ashx?multiverseid={}&type=card", multiverse_id)
}

impl Model {
    fn add_sorted_pick(&mut self, card:Card) {
        while self.sorted_picks.len() <= card.cmc as usize {
            self.sorted_picks.push(vec![]);
        }
        let mut idx = 0;
        for existing in &self.sorted_picks[card.cmc as usize] {
            if existing.name < card.name {
                idx+=1;
            } else {
                break;
            }
        }
        if idx < self.sorted_picks[card.cmc as usize].len() {
            self.sorted_picks[card.cmc as usize].insert(idx, card);
        } else {
            self.sorted_picks[card.cmc as usize].push(card);
        }
    }

    fn pick_card(&mut self, name:String) {
        for card_idx in 0..self.pack.cards.len() {
            let card = self.pack.cards[card_idx].clone();
            if card.name == name {
                self.add_sorted_pick(card);
                self.pack.cards.remove(card_idx);
                self.pack.num_picks += 1;
                if self.pack.num_picks>=PICKS_PER_PACK {
                    self.generate_next_pack();
                }
                break;
            }
        }
    }

    fn generate_next_pack (&mut self) { 
        self.set_generator.progress_pack();
        if self.set_generator.pack_number < NUM_PACKS {
            self.pack = self.set_generator.generate_pack();
        } else {
            self.pack = Pack{cards:vec![], num_picks:0};
        }
    }

    fn produce_pack_header(&self) -> Html {
        if self.set_generator.pack_number >= NUM_PACKS{
            html!{}
        } else {
            html!{
                <>
                <h2> {"Pack "} {self.set_generator.pack_number + 1} </h2>

                <div class="container my-3 bg-light">
                    <div class="col-md-12 text-center">
                        <button type="button" disabled={self.selected.len()<PICKS_PER_PACK as usize} class="btn btn-primary" onclick=self.link.callback(|_| Msg::Confirm())>{"Choose"}</button>
                    </div>
                </div>
                </>
            }
        }
    }

    fn maybe_export_button(&self) -> Html {
        if self.set_generator.pack_number < NUM_PACKS{
            html!{}
        } else {
            html!{
                <>
                <div class="container my-3 bg-light">
                    <div class="col-md-12 text-center">
                        <button type="button" class="btn btn-success" onclick=self.link.callback(|_| Msg::Export())>{"Export to Clipboard"}</button>
                    </div>
                </div>
                </>
            }
        }
    }

    fn draft_screen(&self) -> Html {
        html! {
            <>
            {self.produce_pack_header()}
            <div class="d-flex flex-row flex-wrap px-2 mt-2 bg-light">
                // <button onclick=self.link.callback(|_| Msg::Add(1))>{ "+1" }</button>
                // <button onclick=self.link.callback(|_| Msg::Add(2))>{ "+2" }</button>
                { 
                    for self.pack.cards.iter().map( |e| html!{
                        <CardDisplay  onsignal=self.link.callback(|display| Msg::Select(display)) name=&e.name url=&e.img_url selected=&e.selected/>
                    })
                }
            </div>
            <hr/>
            <h2> {"Deck: "} </h2>
            {self.maybe_export_button()}
            <div class="d-flex container-fluid deck-viewer bg-light">
                <div class="row px-2 mt-2 flipped">
                
                    // <button onclick=self.link.callback(|_| Msg::Add(1))>{ "+1" }</button>
                    // <button onclick=self.link.callback(|_| Msg::Add(2))>{ "+2" }</button>
                    { 
                        for self.sorted_picks.iter().map(|pick_column| html!{
                         
                            <div class="col-xs-4">
                            { 
                                for pick_column.iter().map(|e| html!{
                                    <div class="picked-card-container">
                                        <img class="picked-card shadow-sm mx-1 mt-1 mb-1" src=e.img_url alt=e.name/>
                                    </div>
                                })
                            }
                            </div>
                        })
                    }
                </div>
            </div>
            </>
        }
    }

    fn get_random_set_num (&self) -> i16 {
        return NUM_PACKS-self.get_unassigned_packs_num()
    }

    fn get_unassigned_packs_num (&self) -> i16 {
        let mut sum = 0;
        for set in &self.setup_info.sets {
            sum+=set.1
        }
        return sum
    }

    fn get_unused_sets (&self, ignored_used_set:String) -> Vec<String>{
        let mut unused_sets = vec![];
        for set in self.set_generator.all_set_json.keys() {
            if set == &ignored_used_set {
                unused_sets.push(ignored_used_set.clone());
                continue;
            }
            let mut used = false;
            for used_set in &self.setup_info.sets {
                if used_set.0 == *set {
                    used = true;
                    break;
                }
            }
            if !used {
                unused_sets.push(set.clone());
            }
        }
        return unused_sets;
    }

    fn setup_screen(&self) -> Html {
        html!{
            <>
            <table class="table table-nonfluid table-bordered table-striped text-center">
                <thead>
                    <tr>
                        <th class="text-center">{"Set"}</th>
                        <th class="text-center">{"Number of Packs"}</th>
                    </tr>
                </thead>
                <tbody>
                    {for self.setup_info.sets.iter().map( |setup_set| {
                        // This unpleasant series of variable declarations courtesy of my inability to get around moving one into each enclosure
                        // and having no other place to declare them
                        let set = setup_set.clone();
                        let set_clone = setup_set.clone();
                        let set_clone_2 = setup_set.clone(); 
                        let set_name = setup_set.0.clone(); 
                        let num_packs = setup_set.clone().1; 
                        let num_packs_clone = setup_set.clone().1; 
                        let unassigned_packs = NUM_PACKS - self.get_unassigned_packs_num(); 
                        html!{
                        <tr>
                            <td class="pt-3-half">  
                            <select name="sets" onchange=self.link.callback(move |e| {
                                match e {
                                    yew::html::ChangeData::Select(el) => {
                                        Msg::ChangeSet(set.0.clone(),el.value())
                                    }
                                    yew::html::ChangeData::Value(_) => {
                                        unreachable!()
                                    }
                                    yew::html::ChangeData::Files(_) => {
                                        unreachable!()
                                    }
                                }
                            })>
                            {for self.get_unused_sets(setup_set.0.clone()).iter().map(move |inner_set| {html!{
                                <option value=inner_set.clone() selected={inner_set.clone() == set_clone.0.clone()}>{inner_set}</option>    
                            }})}
                            </select>
                            {" "}
                            <img src={format!("https://gatherer.wizards.com/Handlers/Image.ashx?type=symbol&set={}&size=small&rarity=r", set_name)} alt=""/>
                            </td>
                            
                            <td class="pt-3-half">
                            <select name="numbers" onchange=self.link.callback(move |e| {
                                match e {
                                    yew::html::ChangeData::Select(el) => {
                                        Msg::ChangeSetNumCards(set_clone_2.0.clone(),el.value().parse().unwrap_or(0))
                                    }
                                    yew::html::ChangeData::Value(_) => {
                                        unreachable!()
                                    }
                                    yew::html::ChangeData::Files(_) => {
                                        unreachable!()
                                    }
                                }
                            })>
                            {for (0..unassigned_packs+1+num_packs_clone).map(move |inner_num| {html!{
                                <option value=inner_num.clone() selected={inner_num.clone().to_string() == num_packs.to_string()}>{inner_num}</option>    
                            }})}
                            </select>
                            </td>
                            <td>
                            <svg width="1em" height="1em" viewBox="0 0 16 16" class={format!("bi bi-x-circle-fill {}", if self.setup_info.sets.len() > 1 {"svg-button"} else {"svg-disabled"})} xmlns="http://www.w3.org/2000/svg" onclick={if self.setup_info.sets.len() > 1 {self.link.callback(move |_| Msg::RemoveSet(set_name.clone()))} else {self.link.callback(|_| Msg::DoNothing())}}>
                                <path fill-rule="evenodd" d="M16 8A8 8 0 1 1 0 8a8 8 0 0 1 16 0zM5.354 4.646a.5.5 0 1 0-.708.708L7.293 8l-2.647 2.646a.5.5 0 0 0 .708.708L8 8.707l2.646 2.647a.5.5 0 0 0 .708-.708L8.707 8l2.647-2.646a.5.5 0 0 0-.708-.708L8 7.293 5.354 4.646z"/>
                            </svg>
                            </td>
                        </tr>
                    }})}
                    <tr>
                    <td></td>
                    <td></td>
                    <td>
                        <button type="button" class="btn btn-primary" onclick=self.link.callback(|_| Msg::AddSet())>
                            <svg width="1em" height="1em" viewBox="0 0 16 16" class="bi bi-plus" fill="white" xmlns="http://www.w3.org/2000/svg">
                                <path fill-rule="evenodd" d="M8 4a.5.5 0 0 1 .5.5v3h3a.5.5 0 0 1 0 1h-3v3a.5.5 0 0 1-1 0v-3h-3a.5.5 0 0 1 0-1h3v-3A.5.5 0 0 1 8 4z"/>
                            </svg>
                        </button>
                    </td>
                    </tr>
                    {if self.get_random_set_num()>0 {html!{
                        <tr>
                            <td class="pt-3-half">{"Random"}</td>
                            <td class="pt-3-half">{self.get_random_set_num()}</td>
                            <td></td>
                        </tr>}
                    }else{html!{}}}
                </tbody>
            </table>
            <div class="container my-3">
                <div class="col-md-12 text-center">
                    <button type="button" class="btn btn-success" onclick=self.link.callback(|_| Msg::StartDraft())>{"Start"}</button>
                </div>
            </div>
            </>
        }
    }
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();
    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        println!("preparse");
        let generator =  SetGenerator::new(serde_json::from_str(SET_JSON_STR).unwrap());
        Self {
            link,
            set_generator:generator.clone(),
            setup_info:SetupInfo{sets:vec![(String::from("MH1"),18)]},
            in_draft:false,
            pack:Pack{cards:vec![], num_picks:0},
            sorted_picks: vec![],
            selected:vec![]
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Select(display) => {
                if self.selected.len()>=PICKS_PER_PACK as usize {
                    let oldest_selected = self.selected.first().unwrap();
                    if self.selected.contains(&display) {
                        // card already selected, do nothing
                    } else {
                        // a different card was selected
                        for card_idx in 0..self.pack.cards.len() {
                            let card = &mut self.pack.cards[card_idx];
                            if card.name == display.name {
                                card.selected = true;
                            }
                            if card.name == oldest_selected.name {
                                card.selected = false;
                            }
                        }
                        self.selected.remove(0);
                        self.selected.push(display);
                    }
                } else {
                    for card_idx in 0..self.pack.cards.len() {
                        let card = &mut self.pack.cards[card_idx];
                        if card.name == display.name {
                            card.selected = true;
                        }
                    }
                    self.selected.push(display);
                }
            }
            Msg::Confirm() => {
                for card in self.selected.clone() {
                    self.pick_card(card.name);
                }
                self.selected.clear();
            }
            Msg::Export() => {
                let mut export_map = HashMap::<String, i16>::new();
                for pick_column in &self.sorted_picks {
                    for card in pick_column {
                        if !export_map.contains_key(&card.name) {
                            export_map.insert(card.name.clone(), 0);
                        }
                        export_map.insert(card.name.clone(),export_map[&card.name] + 1);
                    }
                }
                let mut export_list = vec![];
                for card_name in export_map.keys() {
                    export_list.push(format!("{} {}", export_map[card_name], card_name))
                }
                web_sys::window().unwrap().navigator().clipboard().write_text(&export_list.join("\n"));
            }
            Msg::StartDraft() => {
                self.in_draft = true;
                self.set_generator.init_from_setup_packs(&self.setup_info.sets, self.get_random_set_num());
                self.pack = self.set_generator.generate_pack();
            }
            Msg::AddSet() => {
                self.setup_info.sets.push((self.get_unused_sets(String::new()).first().unwrap().clone(), 0));
            }
            Msg::RemoveSet(set_name) => {
                for i in 0..self.setup_info.sets.len() {
                    let set = &self.setup_info.sets[i];
                    if set.0 == set_name {
                        self.setup_info.sets.remove(i);
                        break;
                    }
                }
            }
            Msg::ChangeSet(old_set, new_set) => {
                for i in 0..self.setup_info.sets.len() {
                    let set = &self.setup_info.sets[i];
                    if set.0 == old_set {
                        self.setup_info.sets[i] = (new_set, set.1);
                        break;
                    }
                }
            }
            Msg::ChangeSetNumCards(set_name, num_cards) => {
                for i in 0..self.setup_info.sets.len() {
                    let set = &self.setup_info.sets[i];
                    if set.0 == set_name {
                        self.setup_info.sets[i] = (set_name, num_cards);
                        break;
                    }
                }
            }
            Msg::DoNothing() => {return false}
        }
        true
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        // Should only return "true" if new properties are different to
        // previously received properties.
        // This component has no properties so we will always return "false".
        false
    }

    fn view(&self) -> Html {
        html!{
            <>
            <div id="main">
            {if self.in_draft {
                self.draft_screen()
            } else {
                self.setup_screen()
            }}
            </div>
            <footer class="footer">
            <a href="https://github.com/credman0/supreme-drafter-rs" class="no-decor hover-underline">
            <div class="fl fl-center-y">
            <svg width="18" class="m-r-10" viewBox="0 0 32 32"><path fill="#424950" d="M16 0.4c-8.8 0-16 7.2-16 16 0 7.1 4.6 13.1 10.9 15.2 0.8 0.1 1.1-0.3 1.1-0.8 0-0.4 0-1.6 0-3-4.5 1-5.4-1.9-5.4-1.9-0.7-1.8-1.8-2.3-1.8-2.3-1.5-1 0.1-1 0.1-1 1.6 0.1 2.5 1.6 2.5 1.6 1.4 2.4 3.7 1.7 4.7 1.3 0.1-1 0.6-1.7 1-2.1-3.6-0.4-7.3-1.8-7.3-7.9 0-1.7 0.6-3.2 1.6-4.3-0.2-0.4-0.7-2 0.2-4.2 0 0 1.3-0.4 4.4 1.6 1.3-0.4 2.6-0.5 4-0.5 1.4 0 2.7 0.2 4 0.5 3.1-2.1 4.4-1.6 4.4-1.6 0.9 2.2 0.3 3.8 0.2 4.2 1 1.1 1.6 2.5 1.6 4.3 0 6.1-3.7 7.5-7.3 7.9 0.6 0.5 1.1 1.5 1.1 3 0 2.1 0 3.9 0 4.4 0 0.4 0.3 0.9 1.1 0.8 6.4-2.1 10.9-8.1 10.9-15.2 0-8.8-7.2-16-16-16z"></path></svg>
            <div style="font-size:14px;">{"Github"}</div>
            </div>
            </a>
            </footer>
            
            </>
        }
    }
}

#[wasm_bindgen(start)]
pub fn run_app() {
    utils::set_panic_hook();
    App::<Model>::new().mount_to_body();
}