use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use leptos::*;
use serde::{Deserialize, Serialize};
use std::cmp::{Ord, Ordering};

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct ItemData {
    pub key: String,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Item {
    pub id: usize,
    pub data: ItemData,
    #[serde(default)]
    pub selected: bool,
    #[serde(skip)]
    pub score: Option<u32>,
    #[serde(skip)]
    pub match_indices: Option<Vec<usize>>,
}

impl Item {
    pub fn new(id: usize, data: ItemData) -> Self {
        Self {
            id,
            data,
            score: None,
            match_indices: None,
            selected: false,
        }
    }
}

impl Ord for Item {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self.score, other.score) {
            // Sort by score
            (Some(a), Some(b)) => a.cmp(&b),
            // Items with a score should be above those without
            (Some(_), _) => Ordering::Greater,
            (None, Some(_)) => Ordering::Less,
            // Fallback to the current order of the items
            _ => Ordering::Equal,
        }
    }
}

impl PartialOrd for Item {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

async fn fetch_items(_: ()) -> Vec<Item> {
    include_str!("../countries.txt")
        .lines()
        .enumerate()
        .map(|(i, name)| {
            Item::new(
                i,
                ItemData {
                    key: name.to_string(),
                },
            )
        })
        .collect()
}

#[component]
pub fn App(cx: Scope) -> impl IntoView {
    let (query, set_query) = create_signal(cx, "".to_string());
    let all_items = create_resource(cx, || (), fetch_items);
    let visible_items = move || {
        let mut items = all_items.read(cx).unwrap_or_default();
        let matcher = SkimMatcherV2::default().smart_case();
        items.iter_mut().for_each(
            |item| match matcher.fuzzy_indices(&item.data.key, &query()) {
                Some((score, match_indices)) => {
                    item.score = Some(score as u32);
                    item.match_indices = Some(match_indices);
                }
                None => {
                    item.score = None;
                    item.match_indices = None;
                }
            },
        );
        items.retain(|item| item.score.is_some());
        // TODO: uncomment me to break things!
        // items.sort_by(|a, b| b.cmp(a));
        items.into_iter().take(10).collect::<Vec<Item>>()
    };

    create_effect(cx, move |_| {
        log!("----Start----");
        visible_items().iter().for_each(|item| log!("{:?}", item));
        log!("----End----");
    });
    view! { cx,
        <main class="container">
            <input
                id="query"
                autocomplete="off"
                on:input=move |ev| set_query(event_target_value(&ev))
            />
            <div id="results">
                 <For
                    each=visible_items
                    key=|item| item.id
                    view=move |cx, item| {
                      view! {
                        cx,
                        <button>{item.data.key}</button>
                      }
                    }
                  />
            </div>
        </main>
    }
}
