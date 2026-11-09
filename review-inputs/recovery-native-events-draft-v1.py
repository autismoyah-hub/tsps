"""Unselected conservative observations from already-retained SDK strings."""
import json
MAX_RECORDS=65536
MAX_BYTES=16*1024*1024
MAX_EVENTS=4096
SOURCE='4619ba5838569a68b0f56bb46ab6b2b77c930586'
STAGES=('extract','install-server','ldd','native-load','build','cache','test:zogre-uglug-sale.test.cjs')

def observe(records):
    """Produce bounded indicators only; no success or closure determination.

    SDK records need not preserve original write, stream or line boundaries.
    A line-contained JSON object is only a possible emitted event; absence,
    ordering, truncation and unstructured child output remain unproven.
    Caller must retain independently verified original SDK strings and treat
    exceptions as UNKNOWN, never reconstruct or replace those records.
    """
    if not isinstance(records,list) or len(records)>MAX_RECORDS:raise RuntimeError('RECORD_COUNT')
    total=0; starts={};results={};qualification=[]; suspicious=0; recognized=0
    for record in records:
        if not isinstance(record,str):raise RuntimeError('RECORD_TYPE')
        total+=len(record.encode('utf-8'))
        if total>MAX_BYTES:raise RuntimeError('INPUT_LIMIT')
        for line in record.splitlines():
            try: row=json.loads(line)
            except (ValueError,TypeError):
                if '"event"' in line and any(token in line for token in ('stage-start','stage-result','native-qualification-result')):suspicious+=1
                continue
            if not isinstance(row,dict):continue
            event=row.get('event')
            if event not in ('stage-start','stage-result','native-qualification-result'):continue
            recognized+=1
            if recognized>MAX_EVENTS:raise RuntimeError('EVENT_LIMIT')
            if event=='native-qualification-result':
                status=row.get('status');source=row.get('source')
                if source!=SOURCE or not isinstance(status,str) or len(status)>128:suspicious+=1;continue
                qualification.append(status if status in ('TESTS_PASS_AWAIT_SANDBOX_TERMINAL','SETUP_FAILURE','UNKNOWN','PAYLOAD_NONZERO_OR_TAP_UNCLASSIFIED') else 'OTHER_STATUS')
                continue
            name=row.get('name')
            if name not in STAGES:suspicious+=1;continue
            table=starts if event=='stage-start' else results
            table[name]=table.get(name,0)+1
    return {'kind':'NATIVE_EVENT_OBSERVATIONS_ONLY','sdkRecordCount':len(records),'recognizedEvents':recognized,
            'stageStartCounts':starts,'stageResultCounts':results,'qualificationStatuses':qualification,
            'suspiciousEventLikeLines':suspicious,'interpretation':'UNKNOWN_PENDING_RAW_REVIEW',
            'scope':'SDK-level strings only; no proof of event authenticity, ordering, stream boundaries, completeness, assertion execution, or process closure'}
