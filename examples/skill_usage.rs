use waterscape::{WaterscapeSkill, SkillAction, SkillResponse};

fn main() {
    println!("=== Waterscape Skill Demo ===\n");

    let mut alice_skill = WaterscapeSkill::new("alice");
    let mut bob_skill = WaterscapeSkill::new("bob");

    println!("Skill Metadata:");
    let response = alice_skill.execute(SkillAction::GetMetadata);
    if let SkillResponse::Success { result } = response {
        println!("  Name: {}", result["name"]);
        println!("  Version: {}", result["version"]);
        println!("  Capabilities: {:?}", result["capabilities"]);
    }
    println!();

    println!("Agent Identities:");
    println!("  Alice: {}", alice_skill.public_identity().fingerprint());
    println!("  Bob:   {}", bob_skill.public_identity().fingerprint());
    println!();

    let alice_identity_json = serde_json::to_string(&alice_skill.public_identity()).unwrap();
    let bob_identity_json = serde_json::to_string(&bob_skill.public_identity()).unwrap();

    alice_skill.execute(SkillAction::AddContact {
        identity_json: bob_identity_json,
    });
    bob_skill.execute(SkillAction::AddContact {
        identity_json: alice_identity_json,
    });

    println!("Contacts exchanged successfully!");
    println!();

    println!("Alice's contacts:");
    let response = alice_skill.execute(SkillAction::ListContacts);
    if let SkillResponse::Success { result } = response {
        for contact in result.as_array().unwrap() {
            println!("  - {} ({})", contact["name"], contact["fingerprint"]);
        }
    }
    println!();

    let cover_text = "Hey everyone! Just wanted to share that I found a great new coffee shop downtown.";
    let secret = "CLASSIFIED: New zero-day discovered in MoltX protocol. Patch in progress.";

    println!("Alice encodes a message for Bob:");
    println!("  Cover: \"{}\"", cover_text);
    println!("  Secret: \"{}\"", secret);
    println!();

    let response = alice_skill.execute(SkillAction::Encode {
        recipient_name: "bob".to_string(),
        cover_text: cover_text.to_string(),
        secret_message: secret.to_string(),
    });

    let encoded_text = match response {
        SkillResponse::Success { result } => {
            println!("Encoding successful!");
            result["encoded_text"].as_str().unwrap().to_string()
        }
        SkillResponse::Error { message, code } => {
            panic!("Encoding failed: {} ({})", message, code);
        }
    };
    println!();

    println!("Bob checks for hidden message:");
    let response = bob_skill.execute(SkillAction::CheckHidden {
        text: encoded_text.clone(),
    });
    if let SkillResponse::Success { result } = response {
        println!("  Has hidden message: {}", result);
    }
    println!();

    println!("Bob decodes the message:");
    let response = bob_skill.execute(SkillAction::Decode {
        sender_name: "alice".to_string(),
        text: encoded_text,
    });

    match response {
        SkillResponse::Success { result } => {
            println!("  Decoded: \"{}\"", result["secret_message"]);
        }
        SkillResponse::Error { message, code } => {
            panic!("Decoding failed: {} ({})", message, code);
        }
    }
    println!();

    println!("=== JSON API Demo ===\n");

    let action_json = r#"{
        "action": "CheckHidden",
        "params": {
            "text": "Normal text without hidden data"
        }
    }"#;

    println!("Request: {}", action_json);
    let response = alice_skill.execute_json(action_json);
    println!("Response: {}", response);
    println!();

    println!("=== Group Communication ===\n");

    let charlie_skill = WaterscapeSkill::new("charlie");
    let charlie_identity_json = serde_json::to_string(&charlie_skill.public_identity()).unwrap();

    alice_skill.execute(SkillAction::AddContact {
        identity_json: charlie_identity_json,
    });

    println!("Alice creates a group:");
    let response = alice_skill.execute(SkillAction::CreateGroup {
        group_name: "security-team".to_string(),
        member_names: vec!["bob".to_string(), "charlie".to_string()],
    });

    if let SkillResponse::Success { result } = response {
        println!("  Group '{}' created with {} members", 
            result["group_name"], result["member_count"]);
    }
    println!();

    let group_cover = "Team standup notes: discussed Q1 roadmap and resource allocation.";
    let group_secret = "ALERT: Coordinated attack detected. Initiate defensive measures.";

    println!("Alice sends to group:");
    let response = alice_skill.execute(SkillAction::GroupEncode {
        group_name: "security-team".to_string(),
        cover_text: group_cover.to_string(),
        secret_message: group_secret.to_string(),
    });

    if let SkillResponse::Success { result } = response {
        println!("  Group message encoded successfully");
        println!("  Visible: \"{}\"", result["visible_text"]);
    }
    println!();

    println!("=== Demo Complete ===");
}
