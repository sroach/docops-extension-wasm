mod common;
mod envelope;
mod types;

use wasm_bindgen::prelude::*;

/// Runs once when the wasm module is loaded. Routes Rust panics to
/// console.error with a real message + stack trace instead of an opaque
/// "unreachable executed" trap.
#[wasm_bindgen(start)]
fn init_panic_hook() {
    console_error_panic_hook::set_once();
}

/// Single entry point called from JavaScript, for every visualization type.
/// Kept as `generate_svg` for backwards compatibility with existing HTML —
/// dispatch on type now happens internally via envelope + types::render.
#[wasm_bindgen]
pub fn generate_svg(input: &str) -> String {
    let result = envelope::parse_envelope(input).and_then(|env| {
        let mut svg = types::render(&env.viz_type, env.body, &env.controls)?;

        let meta = common::svg::Metadata::from_controls(&env.controls);
        let meta_block = meta.to_rdf_xml();

        if let Some(pos) = svg.find('>') {
            svg.insert_str(pos + 1, &meta_block);
        }

        if let Some(privkey) = env.controls.get("privkey") {
            common::svg::sign_svg(&mut svg, privkey)?;
        }

        Ok(svg)
    });

    result.unwrap_or_else(|e| common::svg::error_svg(&e))
}

#[cfg(test)]
mod demo_samples_tests {
    use super::*;

    #[test]
    fn test_all_demo_samples_render_successfully() {
        let samples = vec![
            // ADR GraphQL
            r#"[docops,adr]
----
visualVersion=2
title= Adopt GraphQL for API Layer
status= Accepted
date= 2024-07-15
context=
- Our REST APIs have become complex with many endpoints
- Mobile clients need to fetch data from multiple endpoints
- Different clients need different data shapes
- We need to reduce over-fetching and under-fetching of data
decision=
- We will adopt GraphQL for our API layer
- We will maintain existing REST endpoints for backward compatibility
- We will implement a gradual migration strategy
- We will use Apollo Server for the GraphQL implementation
consequences=
- More efficient data fetching for clients
- Improved developer experience with self-documenting API
- Potential learning curve for the team
- Need for new tooling and monitoring
participants=
Alex Rivera | API Architect | alex.rivera@example.com | #4F46E5
Jasmine Wong | Frontend Lead | jasmine.wong@example.com | #059669
David Kim | Backend Developer | david.kim@example.com | #D97706
references=
[[https://graphql.org/ GraphQL Official Documentation]]
[[https://www.apollographql.com/docs/ Apollo GraphQL Documentation]]
[[https://engineering.example.com/graphql-best-practices GraphQL Best Practices]]
----"#,
            // ADR Microservices
            r#"[docops,adr]
----
visualVersion=2
theme=dark
title= Adopt Microservices Architecture
status= Superseded
date= 2024-06-01
template=apple
context=
- Our monolithic application is becoming difficult to maintain and scale
- Development teams need to work independently on different parts of the system
- We need to improve deployment frequency and reduce time-to-market
- Different components have different scaling requirements
decision=
- We will gradually migrate from monolith to microservices architecture
- We will use [[https://martinfowler.com/bliki/DomainDrivenDesign.html domain-driven design]] to identify service boundaries
- We will implement an [[https://aws.amazon.com/api-gateway API gateway]] for client communication
- We will use containerization ([[https://www.docker.com/ Docker]]) and orchestration ([[https://kubernetes.io/ Kubernetes]])
consequences=
- Improved scalability and resilience
- Faster development cycles and independent deployments
- Increased operational complexity
- Need for robust service discovery and monitoring
participants=
Michael Chen | Chief Architect | m.chen@example.com | #3B82F6
Sarah Johnson | DevOps Lead | s.johnson@example.com | #10B981
David Wilson | Dev Manager | d.wilson@example.com | #8B5CF6
references=
[[https://martinfowler.com/articles/microservices.html Martin Fowler on Microservices]]
[[https://kubernetes.io/docs/ Kubernetes Docs]]
----"#,
            // Bar Vertical
            r#"[docops,bar]
----
theme=premium
visualVersion=1
title=Monthly Sales Performance
yLabel=Revenue ($k)
xLabel=Month
type=R
vBar=true
---
January | 120.0
February | 334.0
March | 455.0
April | 244.0
May | 256.0
----"#,
            // Gauge
            r#"[docops,gauge]
----
title=Performance Score
min=0
max=100
suffix=%
---
Result | 75.0
----"#,
            // Quadrant Chart
            r#"[docops,quadrant]
----
title=Magic Quadrant Test
xAxis=Effort
yAxis=Impact
leaders=Leaders
challengers=Challengers
visionaries=Visionaries
niche=Niche
---
Item A | 80 | 80 | Cat 1
Item B | 20 | 80 | Cat 2
Item C | 20 | 20 | Cat 1
Item D | 80 | 20 | Cat 2
----"#,
            // Bar Cylinder
            r#"[docops,bar]
----
theme=premium
title=Cylindrical Annual Growth
shape=cylinder
xLabel=Year
yLabel=Growth Rate (%)
---
2021 | 25.0
2022 | 55.0
2023 | 85.0
2024 | 110.0
----"#,
            // Bar Grouped
            r#"[docops,bar]
----
theme=premium
mode=grouped
shape=rect
title=Annual Product Sales Report
yLabel=Sales (USD)
xLabel=Quarters
---
Product A | Q1 | 5000.0
Product A | Q2 | 7000.0
Product A | Q3 | 8000.0
Product A | Q4 | 6000.0
Product B | Q1 | 6000.0
Product B | Q2 | 8000.0
Product B | Q3 | 7000.0
Product B | Q4 | 9000.0
----"#,
            // Bar Stacked
            r#"[docops,bar]
----
theme=premium
title=Regional Revenue Breakdown
mode=stacked
xLabel=Regions
yLabel=Points
---
North | Alpha | 100.0
North | Beta | 150.0
South | Alpha | 120.0
South | Beta | 90.0
East | Alpha | 140.0
East | Beta | 110.0
West | Alpha | 80.0
West | Beta | 130.0
----"#,
            // Pie Donut Budget
            r#"[docops,pie]
----
title=Budget Allocation
shape=donut
legend=true
percentages=true
theme=premium
visualVersion=1
---
Engineering | 40
Marketing | 25
Sales | 20
Operations | 10
Admin | 5
----"#,
            // Pie Traffic
            r#"[docops,pie]
----
theme=premium
visualVersion=1
title=Website Traffic Sources
---
Organic Search | 35
Direct | 25
Referral | 20
Social Media | 12
Email Campaigns | 8
----"#,
            // Badges Status
            r#"[docops,badge]
----
columns=3
gap=8
---
Build|Passing||#2088ff|#28a745
Tests|1,234 Passed||#6f42c1|#28a745
Coverage|94%||#6f42c1|#00d084
Version|v1.4.0||#4b5563|#2563eb
License|MIT||#4b5563|#10b981
Docs|Live||#374151|#6366f1
Made With|Rust||#d34516|#1e2650||#fcfcfc
Release|Stable||#0369a1|#0284c7
Security|Audited||#15803d|#16a34a
----"#,
            // Badges Styles
            r#"[docops,badge]
----
columns=2
gap=10
---
Flat Style|Default|flat|#374151|#3b82f6
Flat Square|Modern|flat-square|#1f2937|#10b981
Plastic|Glossy|plastic|#111827|#8b5cf6
With Icon|GitHub|flat|#24292e|#2ea44f|github
----"#,
            // Combination
            r#"[docops,combination]
----
title=Sales Volume vs Profit Margin
xLabel=Quarter
yLabel=Units Sold
yLabelSecondary=Margin (%)
dualYAxis=true
theme=premium
---
Units Sold | BAR | Q1 | 1200 |  | PRIMARY
Units Sold | BAR | Q2 | 1450 |  | PRIMARY
Units Sold | BAR | Q3 | 1680 |  | PRIMARY
Units Sold | BAR | Q4 | 1920 |  | PRIMARY
Profit Margin | LINE | Q1 | 22.5 |  | SECONDARY
Profit Margin | LINE | Q2 | 24.8 |  | SECONDARY
Profit Margin | LINE | Q3 | 26.2 |  | SECONDARY
Profit Margin | LINE | Q4 | 28.1 |  | SECONDARY
----"#,
            // Line Chart
            r#"[docops,line]
----
title=Server Response Time Trends
subtitle=Frontend vs Backend latency over 6 months
xLabel=Month
yLabel=Latency (ms)
theme=premium
---
Frontend | Jan | 120
Frontend | Feb | 115
Frontend | Mar | 105
Frontend | Apr | 98
Frontend | May | 92
Frontend | Jun | 85
Backend | Jan | 210
Backend | Feb | 195
Backend | Mar | 180
Backend | Apr | 165
Backend | May | 150
Backend | Jun | 140
----"#,
            // Scorecard Release
            r#"[docops,scorecard]
----
theme=premium
title=Software Release v2.4.0 - Feature & Bug Summary
subtitle=Migration from Legacy System to Modern Architecture
---

[before]
title=BEFORE v2.4.0
---
[before.items]
=== Feature Status
Dark Mode Theme | Missing feature affecting user experience
Multi-language Support | Not available, limiting global reach
Advanced Search Filters | Basic search only, slow performance
=== Known Issues
Login timeout issues | Users frequently logged out
Memory leaks in dashboard | System becomes slow over time
Database connection drops | Intermittent connection failures
---

[after]
title=AFTER v2.4.0
---
[after.items]
=== New Features Added
Dark Mode Theme | Implemented with user preference saving
Multi-language Support | Added 12 languages with automatic detection
Advanced Search Filters | Fast indexing with multiple filter options
=== Bugs Resolved
Login timeout issues | Session management completely rewritten
Memory leaks in dashboard | React components optimized, memory usage -67%
Database connection drops | Connection pooling and retry logic implemented
----"#,
            // Scorecard Dark
            r#"[docops,scorecard]
----
theme=dark
title=Infrastructure Modernization Scorecard
subtitle=Monolith to Cloud Native Kubernetes Migration
---

[before]
title=BEFORE MIGRATION
---
[before.items]
=== Architecture & Scaling
Monolithic App | Difficult to scale individual components
Manual Deployments | High risk releases taking several hours
=== Reliability
Single Point of Failure | Database outages impact all services
Slow Incident Recovery | Mean time to recovery over 2 hours
---

[after]
title=AFTER MIGRATION
---
[after.items]
=== Architecture & Scaling
Microservices on K8s | Auto-scaling pods based on traffic demand
GitOps CI/CD | Automated zero-downtime canary deployments
=== Reliability
Multi-Zone Redundancy | Self-healing cluster with 99.99% uptime
Automated Failover | Instant failover with MTTR under 5 minutes
----"#,
            // Gherkin
            r#"[docops,gherkin]
----
theme=premium
---
Feature: User Authentication
  Scenario: Successful Login
    Given the user is on the login page
    When they enter valid credentials
    Then they should be redirected to dashboard
----"#,
            // Steps
            r#"[docops,steps]
----
title=Customer Onboarding Journey
subtitle=Steps to welcome and activate new customers
footer=Conversion rate improved +15.4% YoY
---
Order | Title | Description | Color | Tag
1 | Sign Up | Customer creates an account | #6EAEFF |
2 | Verification | Identity and email verification | #69DEE5 |
3 | Profile Setup | Complete preferences and settings | #F8BC95 | +12%
4 | First Use | Guided walkthrough of key features | #D9AEF8 |
5 | Engagement | Regular usage and feedback loop | #B0A5FB |
----"#,
            // Recipe
            r#"[docops,recipe]
----
Chocolate Avocado Cake
yield= 8 servings
prep= 20 minutes
cook= 35 minutes
tags= vegan, dessert, healthy
summary= A rich, moist chocolate cake that uses ripe avocado instead of butter or oil for a creamy texture and healthy fats.
ingredients=
- 2 large ripe avocados
- 2 cups all-purpose flour
- 1 cup unsweetened cocoa powder
- 1.5 cups organic cane sugar
- 2 tsp baking soda
- 1 tsp salt
- 2 cups water
- 2 tbsp white vinegar
steps=
1. Preheat oven to 350F (175C) and grease two 8-inch cake pans.
2. Mash avocados until completely smooth in a large bowl.
3. Whisk in all wet ingredients until well combined.
4. Sift in dry ingredients and fold gently until no lumps remain.
5. Divide batter between pans and bake for 30-35 minutes.
notes=
- The avocado flavor completely disappears once baked!
- Best served with a dark chocolate ganache or fresh berries.
----"#,
        ];

        for (i, sample) in samples.iter().enumerate() {
            let svg = generate_svg(sample);
            assert!(
                !svg.contains("DocOps Error") && !svg.contains("class=\"docops-error\""),
                "Sample {} failed to render: {}",
                i,
                svg
            );
            assert!(
                svg.starts_with("<svg"),
                "Sample {} produced invalid SVG output",
                i
            );
        }
    }
}

#[cfg(test)]
mod metadata_tests {
    use super::*;

    #[test]
    fn test_metadata_injection_defaults() {
        let input = "[docops,badge] ---- Label | Message ----";
        let svg = generate_svg(input);
        let expected_date = chrono::Local::now().format("%Y-%m-%d").to_string();

        assert!(svg.contains("<metadata>"));
        assert!(svg.contains("<dc:creator>DocOps.io</dc:creator>"));
        assert!(svg.contains("<dc:rights>MIT License</dc:rights>"));
        assert!(svg.contains("<dc:source>https://roach.gy</dc:source>"));
        assert!(svg.contains(&format!("<dc:date>{}</dc:date>", expected_date)));
    }

    #[test]
    fn test_metadata_injection_custom_override_prevention() {
        let input = "[docops,badge, creator=Jane Doe, date=2026-12-25] ---- Label | Message ----";
        let svg = generate_svg(input);
        let expected_date = chrono::Local::now().format("%Y-%m-%d").to_string();

        assert!(svg.contains("<metadata>"));
        assert!(svg.contains("<dc:creator>Jane Doe</dc:creator>"));
        // Date should NOT be overridden, should be system date
        assert!(svg.contains(&format!("<dc:date>{}</dc:date>", expected_date)));
        assert!(!svg.contains("<dc:date>2026-12-25</dc:date>"));
        // Defaults should still be there for others
        assert!(svg.contains("<dc:rights>MIT License</dc:rights>"));
    }

    #[test]
    fn test_digital_signature_injection() {
        // 32-byte hex private key (64 characters)
        let priv_key = "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f";
        let input = format!(
            "[docops,badge, privkey={}] ---- Label | Message ----",
            priv_key
        );
        let svg = generate_svg(&input);

        assert!(svg.contains("<dc:signature"));
        assert!(svg.contains("sha256-ed25519:"));

        // Ensure it's not the placeholder anymore
        assert!(!svg.contains("SIGNATURE_PLACEHOLDER"));

        // The signature should be a base64 string (88 or 86 chars for Ed25519 signature of 64 bytes)
        // Ed25519 signature is 64 bytes. Base64 of 64 bytes is (64/3) * 4 = 85.33 -> 88 characters.
        // Let's just check it contains a reasonable length signature or at least doesn't contain the placeholder.
    }

    #[test]
    fn test_signature_verification() {
        use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
        use ed25519_dalek::{Signature, SigningKey, Verifier};
        use sha2::{Digest, Sha256};

        let priv_key_hex = "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f";
        let input = format!(
            "[docops,badge, privkey={}] ---- Label | Message ----",
            priv_key_hex
        );
        let svg = generate_svg(&input);

        // 1. Extract signature from SVG
        let sig_prefix = "sha256-ed25519:";
        let start = svg.find(sig_prefix).expect("Signature prefix not found") + sig_prefix.len();
        let end = svg[start..].find('"').expect("Closing quote not found") + start;
        let sig_base64 = &svg[start..end];
        let sig_bytes = BASE64.decode(sig_base64).expect("Invalid base64 signature");
        let signature = Signature::from_slice(&sig_bytes).expect("Invalid signature bytes");

        // 2. Prepare SVG for verification (replace signature with placeholder)
        let svg_for_hash = svg.replace(sig_base64, "SIGNATURE_PLACEHOLDER");

        // 3. Hash
        let mut hasher = Sha256::new();
        hasher.update(svg_for_hash.as_bytes());
        let hash = hasher.finalize();

        // 4. Verify
        let key_bytes = hex::decode(priv_key_hex).unwrap();
        let signing_key = SigningKey::from_bytes(&key_bytes.try_into().unwrap());
        let verifying_key = signing_key.verifying_key();

        verifying_key
            .verify(&hash, &signature)
            .expect("Signature verification failed");
    }
}
