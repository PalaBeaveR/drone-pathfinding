mod utils;

use std::{
    cell::RefCell, collections::HashMap, rc::Rc,
    sync::Mutex, task::Poll,
};

use async_recursion::async_recursion;
use futures::Future;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;
use web_sys::{
    console, window, CustomEvent, CustomEventInit,
};

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc =
    wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

#[derive(Serialize, Deserialize, Clone, Copy)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

impl Point {
    pub fn distance(&self, other: &Point) -> f32 {
        (((other.x - self.x).pow(2)
            + (other.y - self.y).pow(2)) as f32)
            .sqrt()
    }
}

pub struct Frame {
    fired: RefCell<bool>,
}

impl Frame {
    pub fn new() -> Frame {
        Frame {
            fired: RefCell::new(false),
        }
    }
}

impl Future for Frame {
    type Output = ();

    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Self::Output> {
        if *self.fired.borrow() {
            Poll::Ready(())
        } else {
            *self.fired.borrow_mut() = true;
            let waker_clone = cx.waker().clone();
            window()
                .unwrap()
                .request_animation_frame(
                    Closure::once_into_js(move || {
                        waker_clone.wake();
                    })
                    .as_ref()
                    .unchecked_ref(),
                )
                .unwrap();
            Poll::Pending
        }
    }
}

#[wasm_bindgen]
pub fn find_shortest(
    algorithm: JsValue,
    destinations: JsValue,
) -> JsValue {
    if !algorithm.is_string() {
        alert("Algorithm needs to be a string");
        return JsValue::NULL;
    }

    match serde_wasm_bindgen::from_value::<Vec<Point>>(
        destinations,
    ) {
        Ok(destinations) => {
            let mut visited =
                Vec::with_capacity(destinations.len());
            visited.push(0);

            let mut shortest_path =
                Vec::with_capacity(destinations.len());

            (match algorithm.as_string().unwrap().as_str() {
                "naive" => naive_search,
                "closest" => closest_search,
                _ => {
                    alert("Unknown algorithm");
                    return JsValue::NULL;
                }
            })(
                &destinations,
                &mut visited,
                &mut shortest_path,
                &mut f32::MAX.clone(),
            );

            serde_wasm_bindgen::to_value(&shortest_path)
                .unwrap()
        }
        Err(_) => {
            alert("Destinations need to be an array of points {x: number, y: number}");
            JsValue::NULL
        }
    }
}

fn naive_search(
    destinations: &Vec<Point>,
    visited: &mut Vec<usize>,
    shortest_path: &mut Vec<usize>,
    shortest_length: &mut f32,
) {
    if visited.len() >= destinations.len() {
        // Reached the end
        let mut length = 0.;
        let mut last_dest = destinations.first().unwrap();
        for dest in visited
            .iter()
            .skip(1)
            .map(|i| destinations.get(*i).unwrap())
        {
            length += last_dest.distance(dest);
            if length > *shortest_length {
                return;
            }
            last_dest = dest;
        }

        *shortest_length = length;

        shortest_path.clear();
        shortest_path.extend(visited.iter());

        return;
    }
    let not_visited: Vec<usize> = (1..destinations.len())
        .filter(|i| !visited.contains(i))
        .collect();

    for i in not_visited {
        visited.push(i);
        naive_search(
            destinations,
            visited,
            shortest_path,
            shortest_length,
        );
        visited.pop();
    }
}

fn closest_search(
    destinations: &Vec<Point>,
    visited: &mut Vec<usize>,
    shortest_path: &mut Vec<usize>,
    shortest_length: &mut f32,
) {
    if visited.len() >= destinations.len() {
        // Reached the end
        shortest_path.extend(visited.iter());

        return;
    }
    let last =
        destinations.get(*visited.last().unwrap()).unwrap();

    let mut not_visited: Vec<(usize, f32)> = (1
        ..destinations.len())
        .filter(|i| !visited.contains(i))
        .map(|i| {
            (i, last.distance(destinations.get(i).unwrap()))
        })
        .collect();

    not_visited
        .sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

    visited.push(not_visited.first().unwrap().0);
    closest_search(
        destinations,
        visited,
        shortest_path,
        shortest_length,
    );
    visited.pop();
}

#[wasm_bindgen]
pub async fn animate_finding_shortest(
    algorithm: JsValue,
    destinations: JsValue,
) -> JsValue {
    if !algorithm.is_string() {
        alert("Algorithm needs to be a string");
        return JsValue::NULL;
    }

    match serde_wasm_bindgen::from_value::<Vec<Point>>(
        destinations,
    ) {
        Ok(destinations) => {
            let mut visited =
                Vec::with_capacity(destinations.len());
            visited.push(0);

            let mut shortest_path =
                Vec::with_capacity(destinations.len());

            (match algorithm.as_string().unwrap().as_str() {
                "naive" => animated_naive_search,
                "closest" => animated_closest_search,
                _ => {
                    alert("Unknown animated algorithm");
                    return JsValue::NULL;
                }
            })(
                &destinations,
                &mut visited,
                &mut shortest_path,
                &mut f32::MAX.clone(),
            ).await;

            serde_wasm_bindgen::to_value(&shortest_path)
                .unwrap()
        }
        Err(e) => {
            alert("Destinations need to be an array of points {x: number, y: number}");
            JsValue::NULL
        }
    }
}

fn send_animation_frame(path: &Vec<usize>) {
    let mut event = CustomEventInit::new();

    window()
        .unwrap()
        .dispatch_event(
            &CustomEvent::new_with_event_init_dict(
                "animationframe",
                event.detail(
                    &serde_wasm_bindgen::to_value(path)
                        .unwrap(),
                ),
            )
            .unwrap(),
        )
        .unwrap();
}

#[async_recursion(?Send)]
async fn animated_naive_search(
    destinations: &Vec<Point>,
    visited: &mut Vec<usize>,
    shortest_path: &mut Vec<usize>,
    shortest_length: &mut f32,
) {
    send_animation_frame(visited);
    Frame::new().await;
    if visited.len() >= destinations.len() {
        // Reached the end
        let mut length = 0.;
        let mut last_dest = destinations.first().unwrap();
        for dest in visited
            .iter()
            .skip(1)
            .map(|i| destinations.get(*i).unwrap())
        {
            length += last_dest.distance(dest);
            if length > *shortest_length {
                return;
            }
            last_dest = dest;
        }

        *shortest_length = length;

        shortest_path.clear();
        shortest_path.extend(visited.iter());

        return;
    }
    let not_visited: Vec<usize> = (1..destinations.len())
        .filter(|i| !visited.contains(i))
        .collect();

    for i in not_visited {
        visited.push(i);
        animated_naive_search(
            destinations,
            visited,
            shortest_path,
            shortest_length,
        )
        .await;
        visited.pop();
    }
}

#[async_recursion(?Send)]
async fn animated_closest_search(
    destinations: &Vec<Point>,
    visited: &mut Vec<usize>,
    shortest_path: &mut Vec<usize>,
    shortest_length: &mut f32,
) {
    if visited.len() >= destinations.len() {
        // Reached the end
        shortest_path.extend(visited.iter());

        return;
    }
    let last =
        destinations.get(*visited.last().unwrap()).unwrap();

    let mut not_visited: Vec<(usize, f32)> = (1
        ..destinations.len())
        .filter(|i| !visited.contains(i))
        .map(|i| {
            (i, last.distance(destinations.get(i).unwrap()))
        })
        .collect();

    for (i, _) in &not_visited {
        visited.push(*i);
        send_animation_frame(visited);
        visited.pop();
        Frame::new().await;
    }

    not_visited
        .sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

    visited.push(not_visited.first().unwrap().0);
    animated_closest_search(
        destinations,
        visited,
        shortest_path,
        shortest_length,
    )
    .await;
    visited.pop();
}
