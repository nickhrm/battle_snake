import json,glob,sys,collections as C,statistics as st
root=sys.argv[1]
H=json.load(open(root+'/entries/hist.json'))['rows']
ME='honey_badger'
pl=lambda s:1 if '1st' in s else 2 if '2nd' in s else 3 if '3rd' in s else 4 if '4th' in s else 0
ALL=[]
for b in ['duel','standard','royale','constrictor']:
    for r in H[b]:
        try: d=json.load(open(f"{root}/raw/{b}_{r['id']}.json"))
        except: continue
        fr=d['frames']
        if len(fr)<3: continue
        lastm=[s for s in fr[-1]['Snakes'] if s['Name']==ME][0]
        turns=[]
        for f in fr:
            for s in f['Snakes']:
                if s['Name']==ME and not (s.get('Death') and s['Death']['Turn']<f['Turn']):
                    turns.append((f['Turn'],s.get('Shout') or '',int(s['Latency'] or 0) if str(s['Latency']).isdigit() else 0))
        gg=[t for t,sh,l in turns if sh=='gg']
        win=pl(r['pl'])==1
        endT=fr[-1]['Turn']
        # false gg = gg, aber danach lebte Gegner noch >X Zuege / Spiel nicht gewonnen
        false_runs=[]
        if gg:
            run=[gg[0]]
            for t in gg[1:]:
                if t-run[-1]<=1: run.append(t)
                else: false_runs.append(run);run=[t]
            false_runs.append(run)
        ALL.append(dict(b=b,id=r['id'][:8],t=r['t'],win=win,p=pl(r['pl']),gg=gg,runs=false_runs,end=endT,died=bool(lastm.get('Death')),dt=(lastm.get('Death') or {}).get('Turn'),turns=turns,opp=r['opp']))
FIX='2026-11-05 10:50'
print('Board | Spiele | mit gg | gg & nicht gewonnen | gg nur am Ende eines Siegs | gg-Lauf endet >3 Zuege vor Spielende (falsch)')
for b in ['duel','standard','royale','constrictor']:
    x=[g for g in ALL if g['b']==b and g['t']>=FIX]
    wg=[g for g in x if g['gg']]
    lost=[g for g in wg if not g['win']]
    # falsch: letzter gg-Lauf endet deutlich vor Spielende, oder Spiel verloren
    false=[g for g in wg if not g['win'] or any(r[-1]<g['end']-3 for r in g['runs'])]
    fruns=sum(1 for g in wg for r in g['runs'] if r[-1]<g['end']-3 or (not g['win']))
    print(b,len(x),len(wg),len(lost),len(wg)-len(false),'falsche Spiele',len(false),'falsche Laeufe',fruns,'Laufzuege',sum(len(r) for g in wg for r in g['runs'] if r[-1]<g['end']-3 or not g['win']))
print('\nVerlorene Spiele mit falschem gg (Abstand letzte gg bis Tod):')
c=C.Counter();v=[]
for g in ALL:
    if g['t']>=FIX and g['gg'] and not g['win']:
        dist=(g['dt'] or g['end'])-g['gg'][-1]; v.append(dist); c[g['b']]+=1
        print(g['b'],g['id'],g['t'][5:16],'gg-Zuege',len(g['gg']),'letzte',g['gg'][-1],'Tod',g['dt'],'Abstand',dist,'gegen',g['opp'][:3])
print(c,'median',st.median(v) if v else None)
# gewonnene mit falschem gg
print('\nWie viele Niederlagen haben gg in den letzten 30 Zuegen vor Tod?')
for b in ['duel','standard','royale','constrictor']:
    x=[g for g in ALL if g['b']==b and g['t']>=FIX and not g['win'] and g['dt']]
    y=[g for g in x if any(g['dt']-t<=30 for t in g['gg'])]
    print(b,len(y),'/',len(x))
