# Learnings: AF eye focus over the face mesh eye regions

## Step 1

- `eyes::mesh_of` rotates only the face crop (`upright_crop`), not the whole
  preview: on the 1616x1080 orientation-8 preview of `_DSC3927.ARW`,
  `faces::upright_rgb` takes 2.06 ms per call and the 122 px crop 0.016 ms
  (release, 20 runs each). The crop is the window `landmarks_of` would cut
  (`face_square` then `window_at` on the upright size), so the points and the
  pose are the same: the ignored `judges_a_face_in_a_real_image` test now
  turns its upright JPEG back to an orientation-6 stored image and checks
  `mesh_of`'s points against `landmarks_of`'s mapped with `point_to_stored`
  (to 1e-3 px) and the pose for equality; it passed on two faces of
  `D:\Photos\tests\2026-09-28-gui-check-jpeg`.
- Plan deviation: in stored coordinates a portrait preview (orientation 6 /
  8) has its eyes standing on end, so
  the box's width is the eye's height. The first run printed `L 11x20` for a
  97 px face of `_DSC3927.ARW` (orientation 8). `eye_region`'s margin and
  `eye_measures`' `edge_width_rel` therefore use the box's / window's
  **longer side** instead of its width; for an upright eye (wider than tall)
  that is the same thing. The CLI prints each window as `WxH`.
- `riffle-cli candidates` reproduces the frozen numbers with the new columns:
  training AUC lap 0.816 / combined 0.852, 93.1% / 91.4%; held-out 0.635 /
  0.754, 89.1% / 95.3%. The per-file lines are saved in
  `D:\Photos\tests\2026-10-08-mesh-eye-focus\` as `training.txt` (the 5
  training folders), `heldout.txt` (the 4 held-out folders), both at 24
  threads, and `timing-1-thread-2026-06-05.txt` (one folder, 1 thread, for
  the mesh time).
- A mesh failure and a mesh whose regions and pose are all `None` print the
  same; no frame of either set looked like that (every faced frame had at
  least one contour region).
- Coverage of the mesh (faced frames = labeled frames here):

  | | training | held-out |
  | --- | --- | --- |
  | faced frames | 406 | 400 |
  | got a mesh | 406 | 400 |
  | got a pose | 406 | 397 |
  | contour regions (of 2 per frame) | 808 / 812 | 797 / 800 |
  | contour regions under 24 px (longer side) | 723 | 646 |
  | frames with both eyes under 24 px | 343 | 305 |
  | frames with either eye under 24 px | 384 | 344 |
  | contour longer side min / p10 / median / p90 / max | 3 / 6 / 14 / 24 / 57 | 3 / 7 / 15 / 31 / 131 |
  | iris regions (of 2 per frame) | 778 | 769 |
  | iris longer side p10 / median / p90 | 4 / 8 / 12 | 4 / 8 / 16 |
  | faces under 60 px (box long side) | 104 | 102 |

  The eye regions are small: the median contour box with its 0.25 margin is
  14-15 px on its longer side, so a 24 px floor (the window's minimum) would
  send about 85% of the frames to the fallback. Step 2 should choose the floor
  from the AUC by bucket rather than reuse 24; the iris windows (median 8 px)
  look too small for variant (e) on most faces.
- |yaw| of the labeled frames with a pose:

  | absolute yaw (deg) | training (pick / reject) | held-out (pick / reject) |
  | --- | --- | --- |
  | 0-15 | 115 (92 / 23) | 129 (111 / 18) |
  | 15-30 | 92 (79 / 13) | 95 (78 / 17) |
  | 30-45 | 90 (70 / 20) | 80 (71 / 9) |
  | 45-60 | 56 (51 / 5) | 48 (41 / 7) |
  | 60+ | 53 (47 / 6) | 45 (39 / 6) |
  | median / p75 / p90 / max | 29.2 / 45.9 / 67.1 / 110.8 | 25.5 / 42.1 / 63.3 / 134.0 |

  Neither 30 nor 45 deg sits in an empty stretch: both have 50-90 labeled
  frames on each side.
- Mesh time per faced file (`eyes::mesh_of`, the crop, the model and the
  pose solve, model plan built before the timing): 1 thread on
  `2026-06-05-focus-sample` (78 faces) mean 28.8 / median 27.8 / p95 33.7 /
  max 36.6 ms; at 24 threads, in the report's second parallel pass, mean
  101.9 / median 101.3 / p95 120.2 ms (training) and 99.8 / 97.9 / 126.4 ms
  (held-out), the cores being shared with the other files' decodes and
  meshes.

