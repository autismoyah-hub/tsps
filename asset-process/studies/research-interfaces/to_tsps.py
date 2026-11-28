"""to_tsps.py COMPONENTS.json OUT.json: turn an iface.py component list (the decoded form, real/*.json or a rebuild)
into the widget list TSPS's server-defined interfaces take (api.registerCustomInterface({groupId, widgets})).

Field names follow TSPS client/widgets/WidgetLoader.ts decodeIf3 and server/plugins/interface/widgetGroup.js.
Static components only: listeners are dropped (TSPS custom interfaces have no script channel of their own)."""
import sys, json
from pathlib import Path


def convert(comps, group):
    out = []
    for c in comps:
        uid = (group << 16) | c['file']
        par = c['parent']
        w = dict(uid=uid, id=uid, childIndex=-1, parentUid=-1 if par is None else (group << 16) | (par & 0xffff),
                 groupId=group, fileId=c['file'], isIf3=True, type=c['kind'], contentType=0,
                 rawX=c['raw_position'][0], rawY=c['raw_position'][1], rawWidth=c['raw_size'][0], rawHeight=c['raw_size'][1],
                 widthMode=c['size_mode'][0], heightMode=c['size_mode'][1],
                 xPositionMode=c['position_mode'][0], yPositionMode=c['position_mode'][1],
                 x=0, y=0, width=0, height=0, scrollX=0, scrollY=0, scrollWidth=c['scroll_size'][0],
                 scrollHeight=c['scroll_size'][1], isHidden=c['hidden'], hidden=c['hidden'], cachedHidden=c['hidden'],
                 rootIndex=-1, cycle=-1, modelFrame=0, modelFrameCycle=0, aspectWidth=1, aspectHeight=1, itemId=-1,
                 itemQuantity=0, flags=c.get('flags', 0), actions=[a for a in c.get('actions', [])])
        k = c['kind']
        if k == 5:
            w.update(spriteId=c['sprites'][0], spriteAngle=c['sprite_angle'], spriteTiling=c['sprite_tiling'],
                     opacity=c['transparency'], borderType=c['outline'], graphicShadow=c['sprite_shadow'],
                     shadowColor=c['sprite_shadow'], flippedV=c['flip_v'], flippedH=c['flip_h'])
        elif k == 4:
            w.update(fontId=c['font'], text=c['text'], lineHeight=c['line_height'], xTextAlignment=c['text_alignment'][0],
                     yTextAlignment=c['text_alignment'][1], textShadowed=c['text_shadow'], textColor=c['colors'][0],
                     color=c['colors'][0])
        elif k == 3:
            w.update(textColor=c['colors'][0], color=c['colors'][0], filled=c['filled'], opacity=c['transparency'])
        elif k == 9:
            w.update(lineWidth=c['line_width'], textColor=c['colors'][0], color=c['colors'][0], lineDirection=c['line_reversed'])
        out.append(w)
    return out


if __name__ == '__main__':
    spec = json.loads(Path(sys.argv[1]).read_text())
    comps = spec['components'] if isinstance(spec, dict) else spec
    group = comps[0]['group'] if 'group' in comps[0] else int(sys.argv[3])
    Path(sys.argv[2]).write_text(json.dumps(dict(groupId=group, widgets=convert(comps, group)), indent=0))
    print(len(comps), 'widgets for group', group)
