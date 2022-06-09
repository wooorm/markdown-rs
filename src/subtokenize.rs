use crate::content::string::string;
use crate::tokenizer::{Code, Event, EventType, TokenType};
use crate::util::{slice_codes, Span};

pub fn subtokenize(events: Vec<Event>, codes: &[Code]) -> Vec<Event> {
    let mut events = events;
    let mut index = 0;

    // println!("before");
    // while index < events.len() {
    //     let event = &events[index];
    //     println!(
    //         "ev1: {:?} {:?} {:?}",
    //         event.event_type, event.token_type, index
    //     );
    //     index += 1;
    // }
    //
    // index = 0;
    //
    // println!("change");

    while index < events.len() {
        let event = &events[index];

        // println!(
        //     "ev2: {:?} {:?} {:?}",
        //     event.event_type, event.token_type, index
        // );

        if event.event_type == EventType::Enter && event.token_type == TokenType::ChunkString {
            let exit = &events[index + 1];

            assert_eq!(
                exit.event_type,
                EventType::Exit,
                "expected `enter` of `{:?}` to be follow by an `exit` event",
                event.token_type
            );
            assert_eq!(
                exit.token_type, event.token_type,
                "expected `exit` of `{:?}` to follow its `enter` event",
                event.token_type
            );

            let subevents = string(
                slice_codes(
                    codes,
                    &Span {
                        start_index: event.index,
                        end_index: exit.index,
                    },
                ),
                event.point.clone(),
                event.index,
            );
            let len = subevents.len();
            // To do: recursion needed?
            events.splice(index..(index + 2), subevents);
            index += len;
        } else {
            index += 1;
        }
    }

    events
}
