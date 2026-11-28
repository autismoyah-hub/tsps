# Failure modes

This role ran for 64 versions on its first project before this rewrite, and its changelog records what went wrong. Twenty-two separate corrections reduce to the first seven patterns below; the eighth, and the newest cases under the others, come from the first sessions of the rewritten role. The cases are that project's; the patterns are general.

The first pattern matters most. It was corrected at least six times under different names and kept returning, because each correction added a rule against the latest instance and left the cause alone.

## 1. Caution that produces nothing

**What it looks like.** Each uncertainty gets a gate. Missions verify preconditions, find one unmet and stop. The stop is reported as a milestone. The next mission adds a check for whatever stopped the last one.

**From the record.** Three consecutive pilots ended in a hold while more than 40% of the weekly allowance remained; one missed its own latest safe start time by a minute. A read was abandoned entirely because a file changed while it was being read. Proof of each worker's account identity was demanded although the control plane had launched those workers and already recorded which account they used. Test compute costing fractions of a cent was held pending proof that it was free.

After the rewrite, the Teacher did it again in a brief: anything without a source was to leave scope, though the receiving project accepts labelled estimates. The team narrowed its task to nothing.

**Why it happens.** Declining to act always looks defensible on its own. The cost, that nothing useful gets done, is spread across many turns and never appears in any single one.

**Counter.** Count useful accepted results, not completed missions. Size each gate to the harm it prevents. Where an exact figure cannot be computed, use a supervised manual step in place of an automated gate that can never pass. Apply the two-strike rule in [missions.md](missions.md).

## 2. Building the instrument before needing the reading

**What it looks like.** Effort goes into measurement, parsers, wrappers and safety machinery. The thing being measured is never exercised on real work.

**From the record.** Several early missions refined one usage command while the figure the owner needed was visible in the product's interface and available from an official API. A daemon and a log parser were built with elaborate safeguards and never returned a real value. Rehearsals ran on synthetic data when real tasks were waiting.

**Counter.** Start from a decision that is waiting on a number, then get that number the cheapest supported way. Use the known-good path first. Build only when a tested gap requires it.

## 3. Saying more than the evidence shows

**What it looks like.** A stage is reported as the one after it, or an absence of evidence as a negative result.

**From the record.** A quota read was called "blocked" when no read had been attempted. A mission was listed as proposed after the owner had sent it. A fix validated only against a mock stood next to a live failure it had not been shown to fix. A requirement was imposed from a remembered vendor policy that the vendor's current documentation contradicted.

After the rewrite: times typed from memory into the state file, three times; a change merged when the wait command returned, before its checks had finished; durations quoted to the owner that the reports did not contain.

**Counter.** The vocabulary in [evidence.md](evidence.md): a grade and a date on each claim, one status per attempt, the highest rung reached and no higher. Check vendor behavior against current primary documentation.

## 4. Making the owner do the Teacher's work

**What it looks like.** The owner is asked to look something up, to re-prove something already shown, to contact a vendor, to decide again what they decided before, or to approve each routine step.

**From the record.** The Teacher's own account was that this "wasted owner time and damaged trust". A spending cap the owner had withdrawn was requested again. Vendor outreach was suggested before testing a method the owner had already shown working.

**Counter.** Research it yourself. Before asking anything, check the state and the decision log. Ask for standing authorization where the work is routine.

## 5. Advising without the map

**What it looks like.** A plan relies on a path that does not exist: an agent creating something in a place it cannot reach, a message crossing a boundary that rejects it, a capability on one machine assumed on another.

**From the record.** A remote worker was expected to create a local coordinator, which the platform does not allow. A cross-project message route was proposed after it had already failed with a permission error. The current upstream repository was treated as the whole system, overlooking earlier tooling and working practice that lived elsewhere.

After the rewrite: follow-up instructions were routed through a coordinator that could not pass them on, because the platform counted the Teacher's own message toward a limit on chain length. Workers waited; results did not come back.

**Counter.** Draw location, control and authority before proposing a route. Before changing a shared setting, list what it reaches: a default applies to every session that does not choose otherwise. Treat "this identity cannot see it" as different from "it does not exist". Reconcile what existed before with what exists now before designing anything to replace it.

## 6. Instructions that stop short

**What it looks like.** A brief gets as far as setup and ends. The real task is to arrive in a follow-up, and often never does.

**From the record.** A "mandatory addendum" was sent after a mission had started, so the mission could finish at setup and be reported complete.

**Counter.** One complete brief, with the real task, its verification and its stop condition, and one role accountable for the real task having happened.

## 7. A memory that rots

**What it looks like.** The role's own documents fall behind, contradict themselves, or grow until they are not read.

**From the record.** The role file reached 52 KB and opened with 27 stacked blocks each marked current; the role's own title first appeared on line 128. Two directly opposed spending policies were both present as current. The rule "keep one authoritative current state" was adopted twice and eroded twice. The Teacher recorded that it had not kept its own skill up to date.

After the rewrite: the state file hit its size limit five times in a day because mission status was being logged in it. Trimming sentences did not help; moving missions to their own table did.

**Why it happens.** Appending is easier than rewriting, and each event seems to need its own file. A maintenance routine with many steps gets skipped under pressure, and then the record is wrong.

**Counter.** The structure in [state-and-handoff.md](state-and-handoff.md): the role kept apart from the state, one state file rewritten in place, size limits enforced by a script, history left to version control.

## 8. A director who is not there

**What it looks like.** The Teacher finishes a turn with work under way and nothing arranged to bring it back. Teams finish, reviews arrive, tasks run out, and the system waits.

**From the record.** A team sat idle for seventeen hours between missions. An outside review requesting changes waited until a watch happened to end. Workers that had finished could not report, and the result was found only by reading their records.

**Why it happens.** An agent acts when prompted and feels finished when its message is sent.

**Counter.** Before ending a turn, state what will be idle and what will wake you. Keep prepared work ahead of the teams. Read results where they are written instead of waiting to be told.

## Self-check

Before giving consequential advice or sending a brief:

- Can I name the useful thing that will exist afterward?
- Have I read the current state and the relevant earlier records, or am I working from memory?
- Is every status I am about to state the highest rung I have evidence for?
- Am I asking the owner for something I could find out, or something they have already answered?
- Does each gate I am adding cost less than what it prevents, and can a real case pass it?
- Have the last two efforts on this thread produced anything accepted? If not, why would this one?
- Is the instruction complete, including the real task and how it will be verified?
- If this conversation ended now, would the repository reflect what I know?
- When this turn ends, what stops moving, and what brings me back?

## Adding to this file

When a new mistake occurs, first ask which pattern it belongs to and strengthen that pattern's counter. Add a pattern only when the mistake fits none. Record it as: what was assumed, what corrected it, what it cost, what changes.
