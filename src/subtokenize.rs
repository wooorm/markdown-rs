use crate::content::content::start as content;
use crate::content::string::start as string;
use crate::content::text::start as text;
use crate::tokenizer::{
    Code, Event, EventType, State, StateFn, StateFnResult, TokenType, Tokenizer,
};
use crate::util::{slice_codes, Span};
use std::collections::HashMap;

pub fn subtokenize(events: Vec<Event>, codes: &[Code]) -> (Vec<Event>, bool) {
    let mut events = events;
    let mut index = 0;
    // Map of first chunks its tokenizer.
    let mut head_to_tokenizer: HashMap<usize, Tokenizer> = HashMap::new();
    // Map of chunks to their head and corresponding range of events.
    let mut link_to_info: HashMap<usize, (usize, usize, usize)> = HashMap::new();
    let mut done = true;

    while index < events.len() {
        let event = &events[index];

        // Find each first opening chunk.
        if (event.token_type == TokenType::ChunkString
                || event.token_type == TokenType::ChunkText
                || event.token_type == TokenType::ChunkContent) &&
            event.event_type == EventType::Enter &&
            // No need to enter linked events again.
            event.previous == None
        {
            done = false;
            // Index into `events` pointing to a chunk.
            let mut index_opt: Option<usize> = Some(index);
            // Subtokenizer.
            let mut tokenizer = Tokenizer::new(event.point.clone(), event.index);
            // Substate.
            let mut result: StateFnResult = (
                State::Fn(Box::new(if event.token_type == TokenType::ChunkContent {
                    content
                } else if event.token_type == TokenType::ChunkString {
                    string
                } else {
                    text
                })),
                None,
            );
            // Indices into `codes` of each end of chunk.
            let mut ends: Vec<usize> = vec![];

            // Loop through chunks to pass them in order to the subtokenizer.
            while let Some(index_ptr) = index_opt {
                let enter = &events[index_ptr];
                assert_eq!(enter.event_type, EventType::Enter);
                let span = Span {
                    start_index: enter.index,
                    end_index: events[index_ptr + 1].index,
                };
                ends.push(span.end_index);

                if enter.previous != None {
                    tokenizer.define_skip(&enter.point, span.start_index);
                }

                let func: Box<StateFn> = match result.0 {
                    State::Fn(func) => func,
                    _ => unreachable!("cannot be ok/nok"),
                };

                result = tokenizer.feed(slice_codes(codes, &span), func, enter.next == None);

                if let Some(ref x) = result.1 {
                    if !x.is_empty() {
                        // To do: handle?
                        unreachable!("subtokenize:remainder {:?}", x);
                    }
                }

                index_opt = enter.next;
            }

            // Now, loop through all subevents (and `ends`), to figure out
            // which parts belong where.
            // Current index.
            let mut subindex = 0;
            // Index into subevents that starts the current slice.
            let mut last_start = 0;
            // Counter into `ends`.
            let mut end_index = 0;
            let mut index_opt: Option<usize> = Some(index);

            while subindex < tokenizer.events.len() {
                let subevent = &tokenizer.events[subindex];

                // Find the first event that starts after the end we’re looking
                // for.
                // To do: is this logic correct?
                if subevent.event_type == EventType::Enter && subevent.index >= ends[end_index] {
                    let link = index_opt.unwrap();
                    link_to_info.insert(link, (index, last_start, subindex));

                    last_start = subindex;
                    end_index += 1;
                    index_opt = events[link].next;
                }

                subindex += 1;
            }

            let link = index_opt.unwrap();
            link_to_info.insert(link, (index, last_start, subindex));
            head_to_tokenizer.insert(index, tokenizer);
        }

        index += 1;
    }

    // Now that we fed everything into a tokenizer, and we know which parts
    // belong where, the final task is to splice the events from each
    // tokenizer into the current events.
    // To do: instead of splicing, it might be possible to create a new `events`
    // from each slice and slices from events?
    let mut index = events.len() - 1;

    // To do: this is broken, because it can inject linked events, which point
    // to their links through indices, and this messes with all indices.
    // We should try walking front to end instead, keep a count of the shifted
    // index.
    // It’s a bit complex but should work?
    while index > 0 {
        let slice_opt = link_to_info.get(&index);

        if let Some(slice) = slice_opt {
            let (head, start, end) = *slice;
            // If there’s a slice at this index, it must also point to a head,
            // and that head must have a tokenizer.
            let tokenizer = head_to_tokenizer.get(&head).unwrap();

            // To do: figure out a way that moves instead of clones?
            events.splice(index..(index + 2), tokenizer.events[start..end].to_vec());
        }

        index -= 1;
    }

    (events, done)
}
