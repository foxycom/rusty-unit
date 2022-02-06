#[test]
fn testify_1() {
    testify_monitor::MONITOR.with(|l| l.borrow_mut().set_test_id(1u64));
    let mut trie_0 = trie::Trie::new();
    let mut node_2 = trie::Node::clone(&trie_0);
    let mut usize_0 = trie::Trie::count(&node_2);
    let mut node_1 = trie::Node::default();
    let mut node_0 = trie::Node::default();
    trie::Trie::insert(&mut node_0, node_1);
}
#[test]
fn testify_2() {
    testify_monitor::MONITOR.with(|l| l.borrow_mut().set_test_id(2u64));
    let mut trie_0 = trie::Trie::default();
    let mut trie_1 = trie::Trie::clone(&trie_0);
    let mut keyvalueref_0 = iterator::KeyValueRef::clone(&trie_1);
    let mut keyvalue_0 = iterator::KeyValue::clone(&trie_0);
    let mut keyvalue_1 = iterator::KeyValue::clone(&keyvalue_0);
    trie::Node::serialize(&keyvalue_0, keyvalue_1);
}
#[test]
fn testify_3() {
    testify_monitor::MONITOR.with(|l| l.borrow_mut().set_test_id(3u64));
    let mut trie_0 = trie::Trie::new();
    let mut node_0 = trie::Node::default();
    iterator::TrieIntoIterator::next(&mut node_0);
    let mut trie_1 = trie::Trie::new();
    trie::Node::serialize(&trie_0, trie_1);
}
#[test]
fn testify_4() {
    testify_monitor::MONITOR.with(|l| l.borrow_mut().set_test_id(4u64));
    let mut atomvalue_0 = trie::AtomValue::default();
    let mut keyvalue_0 = iterator::KeyValue::clone(&atomvalue_0);
    let mut node_0 = trie::Node::new(keyvalue_0);
    let mut node_1 = trie::Node::clone(&node_0);
    trie::Trie::into_iter(node_1);
}
#[test]
fn testify_5() {
    testify_monitor::MONITOR.with(|l| l.borrow_mut().set_test_id(5u64));
    let mut atomvalue_0 = trie::AtomValue::default();
    let mut keyvalueref_1 = iterator::KeyValueRef::clone(&atomvalue_0);
    let mut keyvalueref_0 = iterator::KeyValueRef::clone(&atomvalue_0);
    let mut bool_1 = trie::AtomValue::eq(&keyvalueref_0, &keyvalueref_1);
    let mut keyvalue_0 = iterator::KeyValue::clone(&atomvalue_0);
    let mut bool_0 = trie::Trie::ne(&atomvalue_0, &keyvalue_0);
}
#[test]
fn testify_6() {
    testify_monitor::MONITOR.with(|l| l.borrow_mut().set_test_id(6u64));
    let mut node_0 = trie::Node::default();
    let mut atomvalue_0 = trie::AtomValue::default();
    let mut node_2 = trie::Node::terminated(atomvalue_0);
    let mut trie_0 = trie::Trie::default();
    let mut node_1 = trie::Node::clone(&node_0);
    let mut bool_0 = trie::Trie::ne(&node_1, &trie_0);
}
#[test]
fn testify_7() {
    testify_monitor::MONITOR.with(|l| l.borrow_mut().set_test_id(7u64));
    let mut node_0 = trie::Node::default();
    let mut node_2 = trie::Node::default();
    iterator::TrieIntoIterator::next(&mut node_2);
    let mut node_1 = trie::Node::clone(&node_0);
    let mut trie_0 = trie::Trie::clone(&node_1);
    let mut trie_1 = trie::Trie::clone(&trie_0);
}
#[test]
fn testify_8() {
    testify_monitor::MONITOR.with(|l| l.borrow_mut().set_test_id(8u64));
    let mut trie_0 = trie::Trie::new();
    let mut atomvalue_0 = trie::AtomValue::default();
    let mut keyvalueref_0 = iterator::KeyValueRef::clone(&trie_0);
    let mut atomvalue_1 = trie::AtomValue::clone(&keyvalueref_0);
    let mut keyvalue_0 = iterator::KeyValue::clone(&atomvalue_1);
    let mut bool_0 = trie::AtomValue::eq(&keyvalueref_0, &atomvalue_0);
}
#[test]
fn testify_9() {
    testify_monitor::MONITOR.with(|l| l.borrow_mut().set_test_id(9u64));
    let mut node_0 = trie::Node::default();
    let mut atomvalue_0 = trie::AtomValue::clone(&node_0);
    let mut keyvalue_0 = iterator::KeyValue::clone(&node_0);
    let mut node_1 = trie::Node::new(keyvalue_0);
    let mut keyvalue_1 = iterator::KeyValue::clone(&node_1);
    let mut bool_0 = trie::Trie::contains_prefix(&keyvalue_1, atomvalue_0);
    iterator::TrieIntoIterator::next(&mut node_1);
}
#[test]
fn testify_10() {
    testify_monitor::MONITOR.with(|l| l.borrow_mut().set_test_id(10u64));
    let mut atomvalue_0 = trie::AtomValue::default();
    let mut trie_1 = trie::Trie::default();
    let mut trie_0 = trie::Trie::new();
    let mut keyvalueref_0 = iterator::KeyValueRef::clone(&atomvalue_0);
    trie::AtomValue::serialize(&atomvalue_0, keyvalueref_0);
}
