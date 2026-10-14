use flaga::prelude::*;

#[test]
fn edge_trigger_basic() {
    let mut em = FlagEventManager { events: Vec::new(), current_flags: 0 };

    // A requires bits 0 and 1
    em.add_event(0b0000_0011, "A(0&1)".to_string());
    // B requires bit 2
    em.add_event(0b0000_0100, "B(2)".to_string());

    let prev = em.current_flags;
    em.update_flags(0b0000_0001);
    assert!(em.get_active_events().is_empty());
    assert!(em.get_triggered_events(prev).is_empty());

    let prev = em.current_flags;
    em.update_flags(0b0000_0011);
    let triggered = em.get_triggered_events(prev);
    assert_eq!(triggered.len(), 1);
    assert_eq!(triggered[0].event_description, "A(0&1)");

    let prev = em.current_flags;
    em.update_flags(0b0000_0111);
    let triggered = em.get_triggered_events(prev);
    // B becomes active now
    assert_eq!(triggered.len(), 1);
    assert_eq!(triggered[0].event_description, "B(2)");
}

// #[test]
// fn note_strategy_is_not_used_by_flag_event_manager_update_flags() {
//     // FlagEventManager::update_flags currently hardcodes "AllRequired semantics" by:
//     // event.is_active = (current & required) == required
//     //
//     // So strategy is currently ignored there; this test documents current behavior.
//     let mut em = FlagEventManager { events: Vec::new(), current_flags: 0 };
// 
//     let e_any = FlagEvent::new(0b0000_0011, "ANY(0|1)")
//         .with_strategy(TriggerStrategy::AnyRequired);
//     em.events.push(e_any);
// 
//     em.update_flags(0b0000_0001);
// 
//     // Under AnyRequired, should_trigger would be true,
//     // but update_flags uses AllRequired, so is_active remains false.
//     assert!(!em.events[0].is_active);
//     assert!(em.events[0].should_trigger(0b0000_0001)); // the event itself says "true"
// }


#[test]
fn note_strategy_is_not_used_by_flag_event_manager_update_flags() {
    let mut em = FlagEventManager { events: Vec::new(), current_flags: 0 };

    em.events.push(
        FlagEvent::new(0b0000_0011, "ANY(0|1)")
            .with_strategy(TriggerStrategy::AnyRequired)
    );

    em.update_flags(0b0000_0001);

    // update_flags currently uses AllRequired semantics to set is_active
    assert!(!em.events[0].is_active);

    // Demonstrate what AnyRequired WOULD do when an event is enabled:
    let e = FlagEvent::new(0b0000_0011, "x")
        .with_strategy(TriggerStrategy::AnyRequired);
    assert!(e.should_trigger(0b0000_0001));
}