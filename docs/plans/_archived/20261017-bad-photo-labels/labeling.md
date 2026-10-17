# Labeling guide: bad photos in the sample folder

## Purpose

The sample folder holds 1,200 copies of frames drawn from 27 of your
folders. Mark the **bad** ones, and only those. These labels mean **bad**,
not "not the best": they show how well Riffle's cues and scores catch the
frames you would throw away, and how many good frames they would drag along.
Every frame you leave unflagged counts as **not bad**.

## What counts as bad

**Reject = you would never keep this frame on its own.** For example:

- eyes closed at the moment of the shot;
- focus missed on the subject;
- motion blur or camera shake;
- the subject cut off or hidden;
- a face turned away at the decisive moment;
- exposure beyond rescue;
- nothing in the frame.

**Not reasons to reject:**

- a better frame of the same moment exists;
- it is a duplicate;
- it is merely an ordinary photo;
- you would not pick it, but you would not mind keeping it.

When unsure, **do not reject**.

## Setup

1. In the settings, turn **AI analysis of what each photo shows** off
   (`aiEnabled`), so the recommended icon and the ranking rows stay out of
   the way.
2. Open the folder `D:\Photos\tests\2026-10-17-bad-labels\sample`.
3. Sort the strip by **File name**, and clear every filter.
4. Let the scan finish before you start.

The files are named `<nnnn>_<folder>_<file>` and shuffled, so the file-name
order is a random order: judge each frame on its own.

## Keys

- `x` rejects the current frame.
- A reject is sticky: pressing `x` again keeps it. To take one back, go to
  the frame and press `u` (unflag), or undo (`Ctrl+Z`) right after.
- **Do not use `shift+x`**: it rejects every other frame of the current
  frame's burst (copies shot within a second of each other still form one).
- With **Auto-advance** on, `x` moves to the next frame; otherwise move with
  the arrow keys. Either is fine.

## How to judge

- **Look at the picture.** Do not go by the `f` focus mark, the strip's face
  icon or the meta pane: they show the cues being evaluated.
- Spend about 1–2 seconds per frame; 1,200 frames take about 20–40 minutes.
- Use as many sittings as you like. Nothing needs saving: each reject is
  written as you press it, and Riffle reopens the folder at the last file
  you viewed.

## Stopping rule

View **every** frame once, from `0001` to the last. A frame left unflagged
counts as not bad, so skip none. When unsure, do not reject.

## When you are done

Say **"done"**, plus anything odd: frames that would not open, frames you
could not judge, doubts about the definition, and roughly how long it took.
