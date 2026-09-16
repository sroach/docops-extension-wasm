---
name: docops-visuals
description: Generate DocOps visualization DSL blocks for visuals, ADRs, scorecards, badges, buttons, and quadrant visuals when chatting with Claude.
---

# DocOps Visuals Skill

Use this skill when the user asks Claude to create a visual, chart, graph, metric card, architecture decision record, badge, CTA button, KPI summary, or prioritization quadrant.

The goal is to generate copy/paste-ready DocOps DSL blocks.

## Supported visual types

Use these canonical visual headers:

```text
[docops,bar]
[docops,pie]
[docops,line]
[docops,combination]
[docops,badge]
[docops,adr]
[docops,scorecard]
[docops,button]
[docops,quadrant]
[docops,timeline]
[docops,recipe]
[docops,gauge]
```

Also recognize these aliases when interpreting user requests:

```text
[docops,barchart]
[docops,piechart]
[docops,pieslice]
[docops,linechart]
[docops,combo]
[docops,score]
[docops,magic]
[docops,tl]
[docops,food]
```

Prefer the canonical names in generated output.

## General rules

When a user asks for a visual:

1. Pick the best visual type.
2. If the user gave enough data, generate the visual immediately.
3. If critical data is missing, ask one concise clarification question.
4. If the user asks for an example, mockup, or sample, create plausible sample data.
5. Always make the output copy/paste-ready.
6. Use `theme=premium` by default.
7. Use `theme=dark` only when the user requests dark mode.
8. Prefer short labels that fit inside a visual.
9. Use numeric values without commas.
10. Do not use Markdown tables inside DocOps blocks.
11. Keep every DocOps block wrapped with opening and closing `----`.

## Choosing the visual type

Use this mapping:

- Compare categories: bar chart
- Compare categories by group: grouped bar chart
- Show stacked contribution: stacked bar chart
- Show part-to-whole composition: pie or donut chart
- Show trends over time: line chart
- Show count plus percentage/rate: combination chart
- Summarize a technical decision: ADR
- Show compact state/version/status: badge
- Show KPI health: scorecard
- Show a link or call to action: button
- Show prioritization or positioning: quadrant chart
- Show a sequence of historical or project milestones: timeline
- Show a structured recipe card: recipe
- Show a single metric gauge: gauge

## Common configuration keys

Use these keys when relevant:

```text
title=...
subtitle=...
theme=premium
visualVersion=1
width=800
height=600
xLabel=...
yLabel=...
shape=...
mode=...
percentages=true
```

Common themes:

```text
theme=premium
theme=dark
theme=agentic
```

## Bar charts

Use bar charts for category comparisons.

### Simple bar chart

```text
[docops,bar]
----
theme=premium
title=Quarterly Revenue
shape=rect
xLabel=Quarter
yLabel=Revenue
---
Q1 | 125000
Q2 | 148000
Q3 | 172000
Q4 | 196000
----
```

### Cylinder bar chart

```text
[docops,bar]
----
theme=premium
title=Yearly Growth
shape=cylinder
xLabel=Year
yLabel=Growth
---
2021 | 25
2022 | 55
2023 | 85
2024 | 110
----
```

### Grouped bar chart

Use this for multiple series across the same categories.

Row format:

```text
Series | Category | Value
```

Example:

```text
[docops,bar]
----
theme=premium
mode=grouped
shape=rect
title=Annual Product Sales
xLabel=Quarter
yLabel=Sales
---
Product A | Q1 | 5000
Product A | Q2 | 7000
Product A | Q3 | 8000
Product A | Q4 | 6000
Product B | Q1 | 6000
Product B | Q2 | 8000
Product B | Q3 | 7000
Product B | Q4 | 9000
----
```

### Stacked bar chart

Use this for contribution-to-total comparisons.

Row format:

```text
Series | Category | Value
```

Example:

```text
[docops,bar]
----
theme=premium
mode=stacked
title=Regional Contribution
xLabel=Category
yLabel=Points
---
Region North | Alpha | 100
Region North | Beta | 150
Region South | Alpha | 120
Region South | Beta | 90
----
```

## Pie and donut charts

Use pie charts for part-to-whole relationships.

Rules:

- Values must be non-negative.
- Values must sum to more than zero.
- Avoid too many slices.
- Prefer donut charts for modern dashboard-style visuals.

### Basic pie chart

```text
[docops,pie]
----
theme=premium
title=Market Share
---
Product A | 30
Product B | 25
Product C | 20
Product D | 15
Product E | 10
----
```

### Premium pie chart

```text
[docops,pie]
----
theme=premium
visualVersion=1
title=Website Traffic Sources
---
Organic Search | 42
Direct | 27
Referral | 16
Social | 10
Paid | 5
----
```

### Donut chart

```text
[docops,pie]
----
theme=premium
shape=donut
title=Revenue by Region
subtitle=Current fiscal year
---
North America | 1905
Europe | 1302
Asia Pacific | 1440
Latin America | 890
Middle East & Africa | 1279
----
```

## Line charts

Use line charts for trends over time.

### Simple line chart

```text
[docops,line]
----
theme=premium
visualVersion=1
title=Website Traffic
---
Product A | 30
Product B | 25
Product C | 20
Product D | 15
Product E | 10
----
```

### Multi-series line chart

Row format:

```text
Series | X Label | Value
```

Example:

```text
[docops,line]
----
theme=premium
title=Monthly Performance Metrics
width=800
xLabel=Month
yLabel=Score
---
Sales | Jan | 40
Sales | Feb | 70
Sales | Mar | 90
Sales | Apr | 70
Sales | May | 40
Sales | Jun | 30
Marketing | Jan | 22
Marketing | Feb | 33
Marketing | Mar | 44
Marketing | Apr | 55
Marketing | May | 66
Marketing | Jun | 77
----
```

## Combination charts

Use combination charts when showing bars and lines together.

Good use cases:

- Revenue and margin
- Volume and conversion rate
- Users and churn rate
- Incidents and SLA percentage
- Sales volume and profit margin

Row format:

```text
Series | BAR_OR_LINE | X Label | Value | Extra | Axis
```

Where:

- `BAR_OR_LINE` is `BAR` or `LINE`
- `Axis` is `PRIMARY` or `SECONDARY`
- Leave `Extra` blank if unused

Example:

```text
[docops,combination]
----
theme=premium
title=Sales Volume vs Profit Margin
subtitle=Quarterly business performance
xLabel=Quarter
yLabel=Units Sold
yLabelSecondary=Margin (%)
dualYAxis=true
---
Units Sold | BAR | Q1 | 1200 |  | PRIMARY
Units Sold | BAR | Q2 | 1450 |  | PRIMARY
Units Sold | BAR | Q3 | 1680 |  | PRIMARY
Units Sold | BAR | Q4 | 1920 |  | PRIMARY
Profit Margin | LINE | Q1 | 22.5 |  | SECONDARY
Profit Margin | LINE | Q2 | 24.8 |  | SECONDARY
Profit Margin | LINE | Q3 | 26.2 |  | SECONDARY
Profit Margin | LINE | Q4 | 28.1 |  | SECONDARY
----
```

## ADR visuals

Use ADR visuals for Architecture Decision Records, RFC outcomes, governance decisions, and major technical choices.

Supported fields:

```text
title=
status=
date=
context=
decision=
consequences=
participants=
references=
```

Common statuses:

```text
Proposed
Accepted
Approved
Completed
Superseded
Deprecated
Rejected
Draft
```

Preferred participant row format:

```text
Name | Role | email@example.com | #HEXCOLOR | emoji
```

Simpler participant format:

```text
Jane Smith (Architect), John Doe (Developer)
```

ADR links use this syntax:

```text
[[https://example.com Link Title]]
```

Example:

```text
[docops,adr]
----
title= Adopt GraphQL for API Layer
status= Accepted
date= 2024-07-15
context=
- REST APIs have become complex with many endpoints
- Mobile clients need data from multiple endpoints
- Different clients need different data shapes
- The team needs to reduce over-fetching and under-fetching
decision=
- Adopt GraphQL for the API layer
- Maintain existing REST endpoints for backward compatibility
- Implement a gradual migration strategy
- Use Apollo Server for the GraphQL implementation
consequences=
- More efficient data fetching for clients
- Improved developer experience with a self-documenting API
- A learning curve for the engineering team
- New observability and governance requirements
participants=
Alex Rivera | API Architect | alex.rivera@example.com | #4F46E5 | 👨‍💻
Jasmine Wong | Frontend Lead | jasmine.wong@example.com | #059669 | 🎨
David Kim | Backend Developer | david.kim@example.com | #D97706 | 🛠️
references=
[[https://graphql.org/ GraphQL Official Documentation]]
[[https://www.apollographql.com/docs/ Apollo GraphQL Documentation]]
----
```

## Badge visuals

Use badges for compact status, version, build state, release state, compliance, ownership, or quality labels.

Example:

```text
[docops,badge]
----
label=Build
message=Passing
color=#10B981
theme=premium
----
```

More examples:

```text
[docops,badge]
----
label=Release
message=v2.4.1
color=#6366F1
theme=premium
----
```

```text
[docops,badge]
----
label=Security
message=Reviewed
color=#059669
theme=premium
----
```

## Scorecard visuals

Use scorecards for KPI summaries, executive dashboards, engineering health, operational health, maturity models, and project snapshots.

Recommended row format:

```text
Metric | Value | Status | Detail
```

Example:

```text
[docops,scorecard]
----
theme=premium
title=Platform Health Scorecard
subtitle=Current sprint snapshot
---
Availability | 99.98% | Excellent | Above SLO target
Latency | 184ms | Good | Within expected range
Error Rate | 0.08% | Excellent | Below alert threshold
Deployment Frequency | 12/week | Good | Stable delivery cadence
Open Incidents | 1 | Watch | Minor customer impact
----
```

## Button visuals

Use button visuals for calls to action, dashboard links, documentation links, workflow launchers, runbooks, and chat links.

Example:

```text
[docops,button]
----
label=Open Dashboard
url=https://example.com/dashboard
style=primary
theme=premium
----
```

More examples:

```text
[docops,button]
----
label=Start Group Chat
url=https://teams.microsoft.com/l/chat/0/0
style=primary
theme=premium
----
```

```text
[docops,button]
----
label=View Runbook
url=https://example.com/runbook
style=secondary
theme=dark
----
```

## Quadrant charts

Use quadrant charts for prioritization, portfolio mapping, risk/value mapping, impact/effort analysis, and magic-quadrant-style visuals.

Recommended row format:

```text
Label | X Value | Y Value | Size | Category
```

Example:

```text
[docops,quadrant]
----
theme=premium
title=Initiative Prioritization Matrix
xLabel=Implementation Effort
yLabel=Business Impact
xMin=0
xMax=100
yMin=0
yMax=100
quadrantTopLeft=Strategic Bets
quadrantTopRight=Major Investments
quadrantBottomLeft=Low Priority
quadrantBottomRight=Quick Wins
---
AI Support Assistant | 35 | 85 | 18 | Product
Billing Modernization | 75 | 80 | 22 | Platform
Internal Wiki Cleanup | 20 | 25 | 10 | Operations
Search Optimization | 45 | 65 | 16 | Growth
Legacy System Rewrite | 90 | 55 | 20 | Engineering
----
```

## Response style

When responding to the user, prefer just the DocOps block.

If helpful, add one sentence before it.

Good:

```text
Here is the DocOps visual:

[docops,line]
----
theme=premium
title=Weekly Active Users
xLabel=Week
yLabel=Users
---
Users | Week 1 | 12400
Users | Week 2 | 13150
Users | Week 3 | 14220
Users | Week 4 | 15880
----
```

Avoid long explanations unless the user asks for them.

## Clarification behavior

Ask a clarification only when required.

Examples:

User says:

```text
Make a chart for performance.
```

Ask:

```text
What performance metrics should the chart show, and over what time period?
```

But if the user asks for a sample or says to use mock data, generate the chart immediately.

## Complete examples

### Executive KPI scorecard

```text
[docops,scorecard]
----
theme=premium
title=Engineering Delivery Scorecard
subtitle=Q3 operating review
---
Lead Time | 2.4 days | Good | Down 18% quarter over quarter
Deployment Frequency | 18/week | Excellent | Healthy release cadence
Change Failure Rate | 4.2% | Good | Within target threshold
MTTR | 36 min | Excellent | Below one-hour goal
Developer Satisfaction | 8.3/10 | Watch | Slight decline from prior survey
----
```

## Timeline visuals

Use timelines for historical events, release history, or project milestones. The visual automatically handles text wrapping and dynamic card heights.

Row format:

```text
date= ...
text= ...
category= ... (optional)
color= ... (optional hex)
```

Example:

```text
[docops,timeline]
----
title= Space Exploration
subtitle= Milestone Achievements
---
date= April 12, 1961
text= Yuri Gagarin becomes the first human to journey into outer space
category= USSR

date= July 20, 1969
text= Neil Armstrong and Buzz Aldrin become the first humans to land on the Moon
category= USA
color= #3B82F6
----
```

## Recipe visuals

Use recipes for food, drinks, or any structured step-by-step instructions with ingredients.

Supported fields:

```text
yield=
prep=
cook=
tags=
summary=
ingredients= (list with -)
steps= (list with 1.)
notes= (optional list with -)
```

Example:

```text
[docops,recipe]
----
Chocolate Avocado Cake
yield= 8 servings
prep= 20 minutes
cook= 35 minutes
tags= vegan, dessert
summary= A rich, moist chocolate cake using avocado instead of butter.
ingredients=
- 2 large ripe avocados
- 2 cups flour
- 1 cup cocoa powder
steps=
1. Preheat oven to 350F.
2. Mash avocados until smooth.
3. Mix ingredients and bake.
----
```

## Gauge charts

Use gauges for single metrics, health scores, or progress indicators.

Supported keys:

```text
min= (default 0)
max= (default 100)
suffix= (e.g. %)
direction= (normal or inverse)
labels= (comma-separated, e.g. LOW,STABLE,HIGH)
```

Example:

```text
[docops,gauge]
----
theme=premium
title=System Health
subtitle=Overall availability
min=0
max=100
suffix=%
---
Score | 98
----
```

### Product adoption donut

```text
[docops,pie]
----
theme=premium
shape=donut
title=Feature Adoption
subtitle=Active users by primary workflow
---
Automation | 4200
Analytics | 3100
Collaboration | 2600
Integrations | 1800
Administration | 900
----
```

### Delivery trend line chart

```text
[docops,line]
----
theme=premium
title=Delivery Throughput
xLabel=Sprint
yLabel=Completed Items
---
Platform | Sprint 1 | 18
Platform | Sprint 2 | 22
Platform | Sprint 3 | 25
Platform | Sprint 4 | 21
Product | Sprint 1 | 14
Product | Sprint 2 | 18
Product | Sprint 3 | 24
Product | Sprint 4 | 28
----
```

### Revenue and margin combination chart

```text
[docops,combination]
----
theme=premium
title=Revenue vs Gross Margin
subtitle=Quarterly financial performance
xLabel=Quarter
yLabel=Revenue
yLabelSecondary=Margin (%)
dualYAxis=true
---
Revenue | BAR | Q1 | 820000 |  | PRIMARY
Revenue | BAR | Q2 | 910000 |  | PRIMARY
Revenue | BAR | Q3 | 1040000 |  | PRIMARY
Revenue | BAR | Q4 | 1180000 |  | PRIMARY
Gross Margin | LINE | Q1 | 61.5 |  | SECONDARY
Gross Margin | LINE | Q2 | 63.2 |  | SECONDARY
Gross Margin | LINE | Q3 | 64.1 |  | SECONDARY
Gross Margin | LINE | Q4 | 66.8 |  | SECONDARY
----
```

### Architecture decision visual

```text
[docops,adr]
----
title= Adopt Event-Driven Integration Pattern
status= Accepted
date= 2026-09-07
context=
- Multiple services need to react to business events
- Synchronous service calls are increasing coupling
- Teams need clearer ownership of integration contracts
decision=
- Publish domain events for important state changes
- Use a managed event broker for delivery
- Define versioned event schemas
- Keep synchronous APIs for request-response workflows
consequences=
- Improved decoupling between services
- Better scalability for event consumers
- Additional operational complexity
- Need for schema governance and event monitoring
participants=
Avery Stone | Principal Architect | avery.stone@example.com | #4F46E5 | 🧭
Mina Patel | Platform Lead | mina.patel@example.com | #059669 | 🚀
Noah Reed | Backend Engineer | noah.reed@example.com | #D97706 | 🛠️
references=
[[https://martinfowler.com/articles/201701-event-driven.html What do you mean by event-driven?]]
[[https://cloudevents.io/ CloudEvents]]
----
```
```
