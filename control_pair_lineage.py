"""Synthetic finite controls only; no provider, game or actual review capture."""
from copy import deepcopy
from pair_lineage import permits_after, BEFORE_SOURCE, AFTER_SOURCE, OWNER, REVIEWER
expected={'source':BEFORE_SOURCE,'name':'before-test','planSHA256':'p','selectionSHA256':'s','jobId':'j','commandSHA256':'c','terminalReceiptSHA256':'t','classificationSHA256':'r'}
good={**expected,'afterSource':AFTER_SOURCE,'owner':OWNER,'reviewer':REVIEWER,'custody':'TERMINAL_RETAINED','formalTerminal':True,'classification':'ACCEPTED_CAUSAL_BEFORE','targetReached':True,'requiredControlsPassed':True,'closureAccepted':True,'reviewCaptureVerified':True}
assert permits_after(good,expected)
for changes in ({'custody':'ACTIVE'}, {'custody':'AMBIGUOUS'}, {'classification':None},
                {'classification':'SETUP_FAILURE','targetReached':False}, {'jobId':'other'},
                {'terminalReceiptSHA256':'other'}, {'closureAccepted':False}, {'reviewCaptureVerified':False},
                {'source':AFTER_SOURCE}, {'afterSource':BEFORE_SOURCE}):
    row=deepcopy(good);row.update(changes);assert not permits_after(row,expected), changes
assert not permits_after(None,expected)
print('PAIR_CONTROLS_PASS 1 permitted, 11 refused; synthetic only')
