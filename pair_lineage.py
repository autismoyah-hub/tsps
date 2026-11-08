"""Finite pair gate. Evidence authenticity and hash binding are caller duties."""
BEFORE_SOURCE = '85f2568b5dba4e2e7968adb099b0bf8e66e1aae7'
AFTER_SOURCE = '2a58cab282b120dca88f8e3eb005df88d8beac6c'
OWNER = '827f21d7-bd7b-4698-9d4d-e4105ca67d27'
REVIEWER = 'a4d43058-5882-445b-8c6f-398aaa054265'

def permits_after(evidence, expected):
    """Permit only the exact independently classified causal predecessor.

    expected is reconstructed from the reviewed pair, submission/selection and
    actual retained bytes; neither input may be derived from the other alone.
    This function never asserts that an input is authenticated.
    """
    if not isinstance(evidence, dict) or not isinstance(expected, dict): return False
    keys = ('source', 'name', 'planSHA256', 'selectionSHA256', 'jobId',
            'commandSHA256', 'terminalReceiptSHA256', 'classificationSHA256')
    if any(not expected.get(k) or evidence.get(k) != expected[k] for k in keys): return False
    if evidence.get('source') != BEFORE_SOURCE or evidence.get('afterSource') != AFTER_SOURCE: return False
    if evidence.get('owner') != OWNER or evidence.get('reviewer') != REVIEWER: return False
    if evidence.get('custody') != 'TERMINAL_RETAINED' or evidence.get('formalTerminal') is not True: return False
    if evidence.get('classification') != 'ACCEPTED_CAUSAL_BEFORE': return False
    if evidence.get('targetReached') is not True or evidence.get('requiredControlsPassed') is not True: return False
    if evidence.get('closureAccepted') is not True or evidence.get('reviewCaptureVerified') is not True: return False
    return True
