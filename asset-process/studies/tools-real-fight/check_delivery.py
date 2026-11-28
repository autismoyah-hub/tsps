"""Read back every deliverable and check local report links."""
import re
from pathlib import Path
import sys

sys.dont_write_bytecode = True
HERE = Path(__file__).resolve().parent
sys.path.insert(0,str(HERE))
import play_real as play
from PIL import Image

report = (HERE/"REPORT.md").read_text(encoding="utf-8")
links = re.findall(r"\]\(<?([^)>]+)>?\)",report)
for link in links:
    assert (HERE/link).is_file(), link
for name in ["encounter",*play.A]:
    gif = play.OUT/(name+".gif")
    sheet = gif.with_suffix(".png")
    expected = play.scene(name)[1]//5
    with Image.open(gif) as im:
        assert im.size == (608,432) and im.n_frames == expected
        for i in range(im.n_frames):
            im.seek(i)
            im.load()
            assert im.info["duration"] == 100
    with Image.open(sheet) as im:
        im.load()
        assert im.size == (1216,648)
    assert "../../render/fight-real-tekton/"+gif.name in links
    assert "../../render/fight-real-tekton/"+sheet.name in links
    print(name, expected)
print(len(links), "links resolved")

