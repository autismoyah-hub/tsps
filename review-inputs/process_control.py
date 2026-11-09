#!/usr/bin/env python3
"""Bounded POSIX group runner; detached descendant closure is unknown."""
import os, resource, selectors, signal, subprocess, time


def run_group(argv, *, cwd, env, stdout_path, stderr_path, timeout_seconds,
              grace_seconds=2, output_limit=1048576, file_limit=None):
    if os.name != 'posix' or not hasattr(os, 'WNOWAIT'): raise RuntimeError('POSIX waitid WNOWAIT required')
    if not argv or not 0 < timeout_seconds <= 1800 or not 0 < grace_seconds <= 30 or not 0 <= output_limit <= 67108864:
        raise ValueError('invalid invocation bounds')
    if file_limit is not None and not 1 <= file_limit <= 1073741824: raise ValueError('invalid file limit')
    counts={'stdout':0,'stderr':0}; errors=[]; truncated=False; timed_out=False; incomplete=False
    process=None; sel=None; registered=[]; leader_exited=False; group_state='unknown'; exit_code=None
    with open(stdout_path,'xb') as stdout, open(stderr_path,'xb') as stderr:
        def limit_files():
            if file_limit is not None: resource.setrlimit(resource.RLIMIT_FSIZE,(file_limit,file_limit))
        process=subprocess.Popen(argv,cwd=cwd,env=env,stdin=subprocess.DEVNULL,stdout=subprocess.PIPE,stderr=subprocess.PIPE,start_new_session=True,shell=False,preexec_fn=limit_files)
        def signal_group(sig):
            try:
                # waitid without WNOWAIT must never be used before final signaling.
                os.waitid(os.P_PID,process.pid,os.WEXITED|os.WNOHANG|os.WNOWAIT)
                if os.getpgid(process.pid)!=process.pid or os.getsid(process.pid)!=process.pid:
                    errors.append('group_identity_mismatch'); return
                os.killpg(process.pid,sig)
            except (ChildProcessError,ProcessLookupError): errors.append('group_identity_unknown')
            except OSError as e: errors.append('signal_'+type(e).__name__)
        def exited_without_reaping():
            result=os.waitid(os.P_PID,process.pid,os.WEXITED|os.WNOHANG|os.WNOWAIT)
            return result is not None
        try:
            sel=selectors.DefaultSelector()
            for name,pipe,target in [('stdout',process.stdout,stdout),('stderr',process.stderr,stderr)]:
                os.set_blocking(pipe.fileno(),False); sel.register(pipe,selectors.EVENT_READ,(name,target)); registered.append(pipe)
            start=time.monotonic(); stopping=None; killed=False
            while True:
                now=time.monotonic()
                if not leader_exited: leader_exited=exited_without_reaping()
                if stopping is None and now-start>=timeout_seconds:
                    timed_out=True; stopping=now; signal_group(signal.SIGTERM)
                if stopping is None and leader_exited:
                    stopping=now; signal_group(signal.SIGTERM)
                if stopping is not None and not killed and now-stopping>=grace_seconds:
                    signal_group(signal.SIGKILL); killed=True
                if stopping is not None and killed and now-stopping>=2*grace_seconds: break
                if leader_exited and not sel.get_map(): break
                for key,_ in sel.select(timeout=.05):
                    name,target=key.data
                    try:
                        block=os.read(key.fd,65536)
                        if not block:
                            sel.unregister(key.fileobj); key.fileobj.close(); continue
                        before=counts[name]; counts[name]+=len(block)
                        target.write(block[:max(0,output_limit-before)])
                        if counts[name]>output_limit:
                            truncated=True
                            if stopping is None: stopping=time.monotonic(); signal_group(signal.SIGTERM)
                    except (OSError,ValueError) as e:
                        errors.append('drain_'+type(e).__name__)
                        try: sel.unregister(key.fileobj)
                        except Exception as e2: errors.append('unregister_'+type(e2).__name__)
                        try: key.fileobj.close()
                        except Exception as e2: errors.append('close_'+type(e2).__name__)
                        if stopping is None: stopping=time.monotonic(); signal_group(signal.SIGTERM)
        except Exception as e:
            errors.append('postspawn_'+type(e).__name__)
        finally:
            # Leader is unreaped here, keeping its PID/PGID reserved for signaling.
            signal_group(signal.SIGKILL)
            if sel is not None:
                try: incomplete=bool(sel.get_map())
                except Exception as e: errors.append('selector_state_'+type(e).__name__); incomplete=True
                try: sel.close()
                except Exception as e: errors.append('selector_close_'+type(e).__name__)
            else: incomplete=True
            for pipe in (process.stdout,process.stderr):
                try: pipe.close()
                except Exception as e: errors.append('pipe_close_'+type(e).__name__)
            try: exit_code=process.wait(timeout=grace_seconds)
            except Exception as e: errors.append('reap_'+type(e).__name__)
            # After reap a numeric PGID might be reused, so absence is observation only.
            try: os.killpg(process.pid,0); group_state='present_after_reap_unverified_identity'
            except ProcessLookupError: group_state='absent_after_reap_observed'
            except OSError as e: errors.append('group_probe_'+type(e).__name__)
        return {'exit_code':exit_code,'timed_out':timed_out,'output_bytes':counts,
                'output_truncated':truncated,'drain_incomplete':incomplete,'drain_errors':errors,
                'group_remaining':group_state!='absent_after_reap_observed','group_state':group_state,
                'detached_descendant_closure':'unknown'}
