# Sittings

A sitting is one review page for the owner, about ten minutes, answered in a browser and saved as a JSON file.

1. Write a spec, e.g. `sample/spec.json` (format in the docstring of `make_sitting.py`). Every item shows the real
   game asset(s) the piece should stand beside, at the same size. Any item that shows one of ours openly must carry
   `"after"` naming the line-up that tests it blind, or the page gives the answer away.
2. `python -I sittings/make_sitting.py SPEC.json` copies the pictures to `REVIEW_DIR/sittings/ID/`, builds the page
   and points `REVIEW_DIR/index.html` at it.
3. Check the page by screenshot before announcing it:
   `msedge --headless=new --window-size=1500,2600 --screenshot=OUT.png file:///REVIEW_DIR/sittings/ID/index.html`.
4. The owner clicks "Save answers". `python -I sittings/collect_sitting.py ID` files the newest download from
   `~/Downloads` into `VERDICTS_DIR/sitting-ID.json` unchanged (a differing second save is kept beside it).
5. Compare the answers with the predictions written before the sitting.

`collect_panel_verdict.py` collects the older single-panel export the same way.

`sample/` holds a spec with three placeholder pictures (plain grey cards, no game content) so the page can be built
and tried: `python -I sittings/make_sitting.py sittings/sample/spec.json --root review-test`.
