# Learnings

## Step 1

- Chose `bottom: 26px` for `.cell span.count`. The cell is 168px tall and the
  image box is its top 144px, so `bottom: 24px` would put the badge's bottom
  edge exactly on the image box's edge (y 144); 26px leaves the same 2px inset
  the stars and flag use from their corners (y 142), well clear of the name
  strip (about y 153-166).
