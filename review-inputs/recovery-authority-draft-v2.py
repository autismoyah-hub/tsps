"""UNSELECTED pure correlation predicate: no session, network, or provider I/O.

Caller supplies complete authenticated captures and the reviewer's retained
CLI response with its independent provenance; a JSON object alone is no proof.
"""
from datetime import datetime, timezone
import hashlib, json

OWNER = '42689c7a-eddd-44ad-b8bb-04bd0f3c70de'
REVIEWER = 'd9664b7d-13e2-464d-81e5-37de82992eb2'
JOB = '6af14d46e6320c66fad7d183'

def stamp(value):
    if not isinstance(value, str): raise ValueError('TIMESTAMP')
    x = datetime.fromisoformat(value.replace('Z', '+00:00'))
    if x.tzinfo is None: raise ValueError('TIMEZONE')
    return x.astimezone(timezone.utc)

def validate(evidence, selection_raw, now):
    """Match one exact action/result/delivery; ambiguity and withdrawal fail.

    Required provenance/completeness flags are claims whose authenticity the
    separately reviewed caller must establish. This does not prove atomicity.
    """
    try:
        if not isinstance(selection_raw, bytes) or not isinstance(now, datetime) or now.tzinfo is None: return 'SELECTION_INVALID'
        selection_sha=hashlib.sha256(selection_raw).hexdigest(); selection=json.loads(selection_raw)
        if not isinstance(selection,dict) or selection.get('owner')!=OWNER or selection.get('reviewer')!=REVIEWER or selection.get('collector')!=OWNER or selection.get('jobId')!=JOB or selection.get('action')!='COLLECT_EXISTING_ONCE': return 'SELECTION_INVALID'
        if evidence.get('authenticated') is not True or evidence.get('completeWindow') is not True: return 'INCOMPLETE_PROVENANCE'
        marker = f'UGLUG_RECOVERY_GO owner={OWNER} reviewer={REVIEWER} job={JOB} selection={selection_sha}'
        actions=evidence['actions']; results=evidence['results']; deliveries=evidence['deliveries']
        if not all(isinstance(x,list) and all(isinstance(item,dict) for item in x) for x in (actions,results,deliveries)): return 'INVALID_CAPTURE'
        matches=[x for x in actions if x.get('session')==REVIEWER and x.get('role')=='assistant' and x.get('kind')=='tool' and x.get('recipient')==OWNER and x.get('text')==marker]
        if len(matches)!=1: return 'ACTION_AMBIGUOUS'
        action=matches[0]; token=action['toolId']; at=stamp(action['timestamp'])
        if not isinstance(token,str) or not token: return 'INVALID_CAPTURE'
        correlated=[x for x in results if x.get('session')==REVIEWER and x.get('toolId')==token]
        if len(correlated)!=1: return 'RESULT_AMBIGUOUS'
        result=correlated[0]; rt=stamp(result['timestamp'])
        # Explicit, retained CLI response is required; normalized tool_result.ok
        # alone contains no exit status and must never be treated as success.
        receipt=result.get('retainedCliResponse')
        if not isinstance(receipt,dict) or receipt.get('verifiedProvenance') is not True or type(receipt.get('exitCode')) is not int or receipt.get('exitCode') != 0 or receipt.get('stdout') != f'sent input to {OWNER}\n': return 'SEND_SUCCESS_UNPROVEN'
        matching=[x for x in deliveries if x.get('session')==OWNER and x.get('role')=='user' and x.get('senderSession')==REVIEWER and x.get('text')==marker]
        if len(matching)!=1: return 'DELIVERY_AMBIGUOUS'
        delivered=stamp(matching[0]['timestamp'])
        if not (at <= rt <= delivered): return 'ORDER_INVALID'
        selected=stamp(selection['selectedUTC']);checked=stamp(evidence['checkedUTC']); now=now.astimezone(timezone.utc)
        if not (selected <= at <= rt <= delivered <= checked <= now): return 'ORDER_INVALID'
        if (now-selected).total_seconds()>900 or (now-checked).total_seconds()>30: return 'STALE_OR_INCOMPLETE'
        if evidence.get('knownHold') is not False or evidence.get('knownRevocation') is not False: return 'KNOWN_OR_AMBIGUOUS_HOLD'
        if evidence.get('reviewWithdrawn') is not False: return 'REVIEW_WITHDRAWN_OR_UNKNOWN'
        return 'CORRELATED_REVIEW_ONLY'
    except (KeyError,TypeError,ValueError,OverflowError,AttributeError):
        return 'INVALID_CAPTURE'
