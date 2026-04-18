# Secure Encrypted Notepad — Launch Plan

**Target launch window:** Tuesday or Wednesday, 14:30–15:00 CEST (UTC+2)

---

## Timing rationale

Best day: Tuesday, Wednesday or Thursday.
Best time: 14:00–17:00 CEST (UTC+2), which covers:
- European afternoon (post-lunch, pre-end-of-day)
- US East Coast morning: 08:00–11:00 EDT (UTC-4)
- US West Coast early morning: 05:00–08:00 PDT (UTC-7)

Avoid Friday afternoons and weekends — posts get buried quickly with fewer active moderators and readers.

---

## Favorable windows in May–June

### Events worth targeting

**World Password Day — Thursday 7 May** ⭐ best possible date
Observed every year on the first Thursday of May — ideal context for an encryption project. On this day r/netsec, r/privacy, and security-focused media actively look for relevant content. A post title referencing World Password Day significantly boosts click-through. Hashtags: `#WorldPasswordDay`, `#PasswordSecurity`.

**Infosecurity Europe — 2–4 June, London**
The largest European security conference. Security communities on HN and Reddit are particularly active that week. A good moment for a post focused on the encryption angle rather than the Rust language.

**Google I/O — typically mid-May**
The broader tech community is active and browsing for new things. Smaller projects can ride the wave of general interest in technology.

### Dates and periods to avoid (Germany / Poland)

| Date | Event | Reason |
|------|-------|--------|
| 1 May | Labour Day (DE + PL) | Large part of the European audience offline |
| 14 May | Ascension Day (DE + PL) | Long weekend in Germany |
| 25 May | Whit Monday (DE + PL) | Public holiday in both countries |
| 25 May | Memorial Day (USA) | Long weekend — falls on the exact same day as Whit Monday |
| 4 June | Corpus Christi (PL + parts of DE) | Public holiday in Poland, some German states |

> Note: US Memorial Day and Whit Monday (DE/PL) fall on the exact same day in 2026 — a double reason to avoid that entire week.

### Recommended windows in May–June

| Window | Why it works |
|--------|-------------|
| **5–9 May** | World Password Day week — best possible context for this project |
| **19–22 May** | After Ascension Day, before the Whit Monday weekend |
| **8–19 June** | After Corpus Christi, before the summer slowdown |

---

## Phase 1 — Pre-launch prep
*Complete by the Monday before launch*

- [ ] **Polish README** — must answer: what algorithm, how keys are stored, threat model, installation instructions
- [ ] **Add security section to README** — explicitly list crypto primitives (e.g. AES-256-GCM, Argon2). This kills ~80% of skeptical HN comments preemptively
- [ ] **Add screenshots or a short GIF demo** — egui app, make it visual. One clear screenshot can double click-through rate
- [ ] **Set up GitHub repo metadata** — topics: `rust`, `encryption`, `notepad`, `egui`, `privacy`. One punchy description sentence. License clearly visible
- [ ] **Add RELEASES.md or CHANGELOG.md** — even for v0.1.0, it signals the project is versioned and taken seriously
- [ ] **Write all platform posts in a draft file** — HN title, Reddit body, Discord message. Write everything before launch day so you can just paste and publish

---

## Phase 2 — Launch day
*Tuesday or Wednesday — starting at 14:30 CEST (UTC+2)*

> Follow this order. Do not skip steps or post out of sequence.

- [ ] **1 · Make the repo public** — do this first, before any posts. Double-check the README renders correctly on GitHub
- [ ] **2 · Post to Hacker News** — title: `Show HN: Secure Encrypted Notepad written in Rust`. Be present for the first 2 hours — early comments define the thread's trajectory
- [ ] **3 · Post to r/rust** — title: `Show r/rust: Secure Encrypted Notepad — written in Rust with egui`. Link directly to the repo, not to the HN thread
- [ ] **4 · Post to r/privacy or r/netsec** — different angle: focus on the encryption, not the language. Mention the threat model it solves
- [ ] **5 · Post in Discord #showcase (Rust-lang)** — short message: what it is, one screenshot, link. Don't paste your HN text verbatim — Discord is casual
- [ ] **6 · Post in Discord #showcase (egui)** — emphasise the egui side: any custom widgets, performance, UI decisions. This audience cares about the UI layer
- [ ] **7 · Post to Lobste.rs** — tags: `rust`, `security`, `encryption`. Lobsters audience is technical and appreciates solid crypto decisions

---

## Phase 3 — Post-launch follow-up
*Wednesday–Thursday after launch*

- [ ] **Respond to every comment within 2 hours of posting** — especially on HN. Engagement in the first window boosts visibility. Answers ≠ defenses — be curious
- [ ] **Submit to This Week in Rust newsletter** — open a PR at `github.com/rust-lang/this-week-in-rust`, add the project under "Crate Updates". ~15k subscribers who are exactly your target audience
- [ ] **Pin a FAQ comment on your Reddit post** — if questions repeat (crypto choice, key storage, roadmap), consolidate answers in one pinned comment
- [ ] **Open a few "good first issue" tickets on GitHub** — projects with beginner-friendly issues attract more stars from lurkers who want to contribute later
- [ ] **Note what worked and what didn't** — where did the most engagement come from? What questions were asked that you didn't anticipate? Useful for your next release

---

## Key reminders

**The most important element of the entire plan** is the security section in the README. On HN, projects with encryption receive immediate skepticism: *"which algorithms, was there an audit, why not library X?"* If the README answers these questions before anyone asks, comments become constructive instead of defensive.

**The post order on launch day is intentional** — HN goes first because it can deliver the largest single burst of traffic and discussion. Reddit immediately after, so both posts are "fresh" in the same time window. Discord and Lobsters can follow 30–60 minutes later without any loss.

**This Week in Rust** is the most commonly skipped step — and yet it's free access to ~15k subscribers who are exactly your target audience. The PR to their repo takes 5 minutes.
