#[cfg(feature = "moltbook")]
use waterscape::{
    Agent, MoltbookConfig, WaterscapeMoltbook,
    moltbook::{MockMoltbookClient, MoltbookPost},
};

#[cfg(feature = "moltbook")]
#[tokio::main]
async fn main() {
    println!("=== Waterscape + Moltbook Integration Demo ===\n");

    let alice = Agent::new("alice");
    let bob = Agent::new("bob");

    println!("Created agents:");
    println!("  Alice: {}", alice.public_identity().fingerprint());
    println!("  Bob:   {}", bob.public_identity().fingerprint());
    println!();

    let mock_client = MockMoltbookClient::new();

    let alice_moltbook = WaterscapeMoltbook::new(alice, mock_client);

    let bob_identity = bob.public_identity();

    let cover_text = "Just finished my morning coffee! ☕ Hope everyone has a great day!";
    let secret = "URGENT: Security vulnerability found in MoltX. Do not disclose publicly.";

    println!("Alice posts to m/general:");
    println!("  Visible: \"{}\"", cover_text);
    println!("  Hidden:  \"{}\"", secret);
    println!();

    let post_id = alice_moltbook
        .send_post("m/general", cover_text, secret, &bob_identity)
        .await
        .expect("Failed to create post");

    println!("Post created with ID: {}", post_id);
    println!();

    println!("Bob scans m/general for hidden messages...");
    println!("(In production, Bob would use his own WaterscapeMoltbook instance)");
    println!();

    println!("=== Demo Complete ===");
    println!();
}

#[cfg(not(feature = "moltbook"))]
fn main() {
    println!("This example requires the 'moltbook' feature.");
    println!("Run with: cargo run --example moltbook_integration --features moltbook");
}
