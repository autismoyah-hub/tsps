"""Unselected conservative resource preflight for pinned frame transport."""
import json
MAX_RECORDS=65536
MAX_INPUT_BYTES=16*1024*1024
MAX_LINE_BYTES=1024*1024
MAX_RELEVANT_EVENTS=65536
MAX_PAIRS=128
MAX_DECLARED_BYTES=16*1024*1024

def validate(lines):
    """Bound work before decoding. Does not prove payload/test completion."""
    if not isinstance(lines,list) or len(lines)>MAX_RECORDS:raise RuntimeError('RECORD_COUNT')
    pairs=set();events=total=declared=0
    for block in lines:
        if not isinstance(block,str):raise RuntimeError('RECORD_TYPE')
        encoded=block.encode('utf-8');total+=len(encoded)
        if total>MAX_INPUT_BYTES:raise RuntimeError('INPUT_LIMIT')
        for line in block.splitlines():
            if len(line.encode('utf-8'))>MAX_LINE_BYTES:raise RuntimeError('LINE_LIMIT')
            try: row=json.loads(line)
            except (ValueError,TypeError):
                if '"child-output-frame"' in line:raise RuntimeError('MALFORMED_FRAME')
                continue
            if not isinstance(row,dict):continue
            event=row.get('event')
            if event not in ('child-output-frame','stage-result'):continue
            if event=='stage-result' and 'outputTransport' not in row:continue
            events+=1
            if events>MAX_RELEVANT_EVENTS:raise RuntimeError('EVENT_LIMIT')
            stage=row.get('stage') if event=='child-output-frame' else row.get('name')
            if not isinstance(stage,str) or not 1<=len(stage)<=128:raise RuntimeError('STAGE_SCHEMA')
            if event=='child-output-frame':
                stream=row.get('stream')
                if stream not in ('stdout','stderr'):raise RuntimeError('STREAM_SCHEMA')
                pairs.add((stage,stream))
            else:
                transport=row.get('outputTransport')
                if not isinstance(transport,dict):raise RuntimeError('METRICS_SCHEMA')
                for stream in ('stdout','stderr'):
                    metrics=transport.get(stream)
                    if not isinstance(metrics,dict) or type(metrics.get('bytes')) is not int or not 0<=metrics['bytes']<=MAX_DECLARED_BYTES or type(metrics.get('frames')) is not int or not 0<=metrics['frames']<=MAX_RELEVANT_EVENTS:raise RuntimeError('METRICS_SCHEMA')
                    declared+=metrics['bytes'];pairs.add((stage,stream))
                    if declared>MAX_DECLARED_BYTES:raise RuntimeError('DECLARED_LIMIT')
            if len(pairs)>MAX_PAIRS:raise RuntimeError('PAIR_LIMIT')
    if not pairs:raise RuntimeError('NO_FRAME_EVENTS')
    return {'inputBytes':total,'relevantEvents':events,'streamPairs':len(pairs),'declaredBytes':declared,'scope':'resource preflight only; decoder and output integrity still required'}
