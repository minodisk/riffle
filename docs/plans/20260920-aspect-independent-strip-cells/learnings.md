# Learnings

## Step 1

- The placeholder grey had to move off the loaded image: with a 144x144 box,
  `background` on `.cell img` would paint 24px grey bands above and below a
  loaded 3:2 thumbnail. `.cell img:not([src])` is exactly "placeholder or
  failed" because `createCell` in `crates/app/ui/src/strip.ts` appends an
  `<img>` with no `src`, sets `src` only when the payload arrives, and
  recreates cells rather than reusing them.
- The visual acceptance criteria (a 3:2 thumbnail looking identical to before,
  a non-3:2 thumbnail filling the cell width/height, the placeholder still
  grey) could not be confirmed by eye from this session, which cannot drive the
  GUI. They follow from `object-fit: contain` keeping the intrinsic aspect and
  centring within the box, but the by-eye check is left to the user.
