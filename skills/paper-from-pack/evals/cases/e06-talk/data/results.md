# Results: retry policies for flaky tests in a toy CI pipeline (synthetic)

Run date 2026-01-15. Plan fixed before the first run. Harness notes: `data/harness-notes.md`.

## 1. Question and answer
Question: do retry policies remove false red builds without hiding real failures? Answer: on a toy suite, retrying every failed test (P1) and retrying only tests flagged as flaky (P2) both cut false red builds sharply; P1 missed 1 of 10 injected real defects, P2 missed none. This is a small controlled study, not evidence about production pipelines.

## 2. Setup
- Suite: 120 tests, of which 18 are flaky by construction (random sleep and a shared port).
- Policies: P0 no retry; P1 retry any failed test up to 2 times; P2 retry up to 2 times only tests on a flaky list built from the previous 20 runs.
- 40 CI runs per policy, 120 runs in total. In 10 of the 40 runs per policy one real defect was injected.
- One machine, 8 cores, containers recreated for every run.

## 3. Results
| Policy | Runs with a false red build | Median wall time (s) | Injected defects missed |
|---|---|---|---|
| P0 | 31 of 40 | 212 | 0 of 10 |
| P1 | 4 of 40 | 268 | 1 of 10 |
| P2 | 6 of 40 | 231 | 0 of 10 |

- P1 raised median wall time by 56 s over P0 (268 s against 212 s); P2 by 19 s (231 s against 212 s).
- The one missed defect under P1 was a test that failed on its first try and passed on retry because the injected defect was timing-dependent.
- The flaky list used by P2 contained 15 of the 18 flaky tests at run 21.

## 4. Limits
- Toy suite of 120 tests; defects were injected, not natural.
- One machine and one container image; no parallel CI workers.
- 40 runs per policy; no confidence intervals computed.
- No cost data of any kind was collected: nothing here measures CI spend, developer time or production pipelines.
- The flaky list for P2 came from the same suite it was evaluated on.

## 5. Not done
Real repositories, other languages, queueing effects, longer histories.
