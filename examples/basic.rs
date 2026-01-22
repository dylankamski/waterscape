use waterscape::{Agent, Waterscape, WaterscapeGroup};

fn main() {
    println!("=== Waterscape Protocol Demo ===\n");

    let alice = Agent::new("alice");
    let bob = Agent::new("bob");

    println!("Created agents:");
    println!("  Alice: {}", alice.public_identity().fingerprint());
    println!("  Bob:   {}", bob.public_identity().fingerprint());
    println!();

    let cover_text = "Hello everyone! Just wanted to share that the weather is beautiful today. \
                      Hope you're all having a great time!";
    let secret_message = "URGENT: Security breach detected. Initiate protocol omega. \
                          Rendezvous at backup location alpha-7.";

    println!("Cover text (visible to humans):");
    println!("  \"{}\"", cover_text);
    println!();

    println!("Secret message:");
    println!("  \"{}\"", secret_message);
    println!();

    let encoded = Waterscape::encode(
        &alice,
        &bob.public_identity(),
        cover_text,
        secret_message,
    )
    .expect("Encoding failed");

    println!("Encoded message length: {} chars", encoded.len());
    println!("Original length: {} chars", cover_text.len());
    println!("Hidden data overhead: {} chars", encoded.len() - cover_text.len());
    println!();

    let visible = Waterscape::visible_text(&encoded);
    println!("Visible text after encoding:");
    println!("  \"{}\"", visible);
    assert_eq!(visible, cover_text);
    println!();

    println!("Has hidden message: {}", Waterscape::has_hidden_message(&encoded));
    println!();

    let decoded = Waterscape::decode(&bob, &alice.public_identity(), &encoded)
        .expect("Decoding failed");

    println!("Bob decoded:");
    println!("  \"{}\"", decoded);
    assert_eq!(decoded, secret_message);
    println!();

    let eve = Agent::new("eve");
    let eve_result = Waterscape::decode(&eve, &alice.public_identity(), &encoded);
    println!("Eve (unauthorized) tries to decode: {:?}", eve_result.is_err());
    println!();

    println!("=== Group Communication ===\n");

    let charlie = Agent::new("charlie");
    let members = vec![
        alice.public_identity(),
        bob.public_identity(),
        charlie.public_identity(),
    ];

    let group = WaterscapeGroup::new("secret-council", &alice, members);
    println!("Created group: {}", group.name());
    println!("Members: {}", group.members().len());
    println!();

    let group_cover = "Team meeting notes: discussed project timeline and resource allocation.";
    let group_secret = "CLASSIFIED: New vulnerability discovered in MoltX protocol. \
                        Do not disclose until patch is ready.";

    let group_encoded = group
        .encode(&alice, group_cover, group_secret)
        .expect("Group encoding failed");

    let group_decoded = group.decode(&group_encoded).expect("Group decoding failed");

    println!("Group message decoded: \"{}\"", group_decoded);
    assert_eq!(group_decoded, group_secret);

    println!("\n=== Demo Complete ===");
}
