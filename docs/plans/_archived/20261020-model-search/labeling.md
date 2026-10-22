# Labeling guide: frames that should not be discarded (the confirmation round)

## Purpose

The sample folder holds 600 copies of frames drawn from 27 of your folders,
none of them a frame you labeled in the two earlier rounds. The question for
each frame is the same as last time, **"OK to discard?"**: if Riffle hid this
frame from what you pick from, would that be fine? It is not "is this frame
bad", and not "would you pick it". The labels check, once, whether the
discard suggestion chosen in the exploration holds up on frames it was not
chosen on.

## What to mark

**Mark a frame when it should not be discarded:**

- you might pick it;
- you would want to see it before deciding;
- you would mind if Riffle had hidden it from what you pick from.

**Leave it unmarked when you would not miss it**, for example:

- a blurred frame, or one with the eyes closed;
- a dead frame of a burst you would never choose;
- a duplicate you would not look at twice.

When unsure, **mark it (keep)**.

## What the folder holds

The folder mixes frames Riffle proposes to discard with frames drawn at
random, and nothing shows which is which: not the name, not the order, not
the folder. Judge each frame on its own.

## Setup

1. In the settings, turn **AI analysis of what each photo shows** off
   (`aiEnabled`), so the recommended icon and the ranking rows are absent.
2. Open the folder `D:\Photos\tests\2026-10-20-model-search\sample`.
3. Sort the strip by **File name**, and clear every filter.
4. Let the scan finish before you start.

The files are named `<nnnn>_<folder>_<file>` and shuffled, so the file-name
order is a random order. No frame has a rating, a flag or a label yet.

## Keys

- **`1`** (rating 1) marks the current frame as **keep**. With
  **Auto-advance** on, it moves to the next frame; otherwise move with the
  arrow keys. Either is fine.
- **`0`** clears the rating, to take a mark back.
- **Do not use**:
  - `p` (Pick): a pick in this folder would train the personal ranking;
  - `x` / `shift+x` (Reject): that was the strict label of an earlier round,
    not this question;
  - any other star rating (`2` to `5`).

## How to judge

- **Look at the picture.** Do not go by the `f` focus mark, the strip's
  icons or the meta pane.
- Spend about 1–3 seconds per frame; 600 frames take about 15–30 minutes.
- Use as many sittings as you like. Nothing needs saving: each rating is
  written as you press it, and Riffle reopens the folder at the last file
  you viewed.
- Do not compare with how you labeled the earlier rounds; judge each frame
  as it is.

## Stopping rule

View **every** frame once, from `0001` to `0600`. A frame left unmarked
counts as OK to discard, so skip none. When unsure, mark it.

## When you are done

Say **"done"**, plus roughly how long it took, and anything odd: frames that
would not open, frames you could not judge, doubts about the definition, and
whether `aiEnabled` was off throughout.
