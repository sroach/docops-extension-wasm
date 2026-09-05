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
    let result = envelope::parse_envelope(input)
        .and_then(|env| types::render(&env.viz_type, env.body, &env.controls));

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
June | 223.0
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
        ];

        for (i, sample) in samples.iter().enumerate() {
            let svg = generate_svg(sample);
            assert!(
                !svg.contains("DocOps Error") && !svg.contains("class=\"docops-error\""),
                "Sample {} failed to render: {}",
                i,
                svg
            );
            assert!(svg.starts_with("<svg"), "Sample {} produced invalid SVG output", i);
        }
    }
}
