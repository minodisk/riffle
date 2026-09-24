# Investigation: camera sequence records vs. the 1 s burst rule

Data: 2134 Sony α7 V (ILCE-7M5) ARWs from one shoot (2026-09-19), read with
ExifTool 13.59. The first half (`_DSC1796`–`2477`) used the mechanical shutter
at 10 fps without pre-capture; the second half used the electronic shutter at
20 fps with pre-capture enabled.

## Tags the α7 V writes

- `ReleaseMode` (MakerNote 0xb049, plain): `Continuous` on 2133 of 2134 files.
  It reflects the drive setting only, so a single press in continuous mode
  also reads `Continuous`; it cannot tell single shots apart.
- `SequenceNumber` (0xb04a, plain int16u): the frame index within one press.
  It equals `SequenceImageNumber` in the enciphered 0x9400 block on every file
  but one.
- `SequenceFileNumber` (enciphered 0x9400 block): restarts at 1 for each press,
  and additionally at the boundary between pre-capture frames and the frames
  after the full press. It restarted while `SequenceNumber` kept counting in 85
  places, all in the pre-capture half and none in the first half.

## Rules compared on the whole folder

| Rule | Bursts | Singles | Widest in-burst gap |
|---|---|---|---|
| Time gap > 1 s (current) | 316 | 23 | 0.988 s |
| `SequenceFileNumber` restarts | 450 | 50 | 0.234 s |
| `SequenceNumber` not consecutive, or gap > 1 s | 416 | 46 | 0.234 s |

## Ground truth from the user

| Files | User | 1 s rule | `SequenceNumber` | `SequenceFileNumber` |
|---|---|---|---|---|
| 2659–2662 | A | A | A | A |
| 2663–2672 (pre-capture) | B | B | B | B |
| 2673–2684 (after full press) | B | B | B | C |
| 2685–2686 (re-press 0.73 s later) | B | B | C | D |
| 2687–2689 (1.17 s later) | C | C | D | E |

A burst in Riffle is one moment, not one shutter press, so the camera's
numbering over-splits. The 1 s rule matched the user's grouping exactly, and
it is kept.
