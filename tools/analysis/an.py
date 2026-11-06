import json,glob,sys,bisect,statistics as st,datetime as dt,collections as C
root=sys.argv[1]
JOIN=dt.datetime(2026,11,5,11,20,tzinfo=dt.timezone.utc).timestamp()
ME='honey_badger'
G=[]
for f in glob.glob(root+'/games/*.json'):
    d=json.load(open(f))
    if not d['frames']: continue
    fr=d['frames']
    if not any(s['N']==ME for s in fr[0]['S']): continue
    d['mode']='duel' if d['n']==2 else ('royale' if d['map']=='royale' or d['ruleset']=='royale' else ('constrictor' if 'constrictor' in d['ruleset'] else 'standard'))
    t=d['created']/1e6; moves=[]; res=None
    for a in fr[1:]:
        ls=[]; me=None; others=[]
        for s in a['S']:
            died_before=s['D'] and s['DT'] is not None and s['DT']<a['T']
            if died_before: continue
            try:l=int(s['L'] or 0)
            except: l=0
            ls.append(l)
            if s['N']==ME: me=(l,s['E'])
            else: others.append((l,s['E']))
        t+= (max(ls) if ls else 0)/1000+0.02
        if me: moves.append(dict(t=t,turn=a['T'],lat=me[0],err=me[1],otherto=sum(1 for l,e in others if l>=500 or e)))
    last=fr[-1]['S']; m=[s for s in last if s['N']==ME][0]
    alive=[s for s in last if not s['D']]
    place=1+sum(1 for s in last if s['N']!=ME and ((s['D'] is None) or (m['D'] and (s['DT'] or 0)>(m['DT'] or 0)))) 
    d.update(moves=moves,start=d['created']/1e6,end=t,win=(not m['D'] and len(alive)==1),place=place,dcause=m['D'],dturn=m['DT'],turns=fr[-1]['T']); del d['frames']
    G.append(d)
print('games',len(G),dt.datetime.fromtimestamp(min(g['start'] for g in G),dt.timezone.utc),dt.datetime.fromtimestamp(max(g['start'] for g in G),dt.timezone.utc))
def conc(t,g): return sum(1 for h in G if h['start']<=t<=h['end'])
for g in G:
    for m in g['moves']: m['c']=conc(m['t'],g)
json.dump(G,open(root+'/processed.json','w'))
def pct(x,p):
    x=sorted(x); return x[min(len(x)-1,int(len(x)*p))] if x else 0
def row(name,ms):
    th=[m['lat'] for m in ms if m['lat']>=300]
    to=sum(1 for m in ms if m['lat']>=500 or m['err'])
    own=sum(1 for m in ms if (m['lat']>=500 or m['err']) and m['otherto']==0)
    print(f"{name:26s} moves={len(ms):6d} TO={to:3d} ({to/max(1,len(ms))*1e4:5.1f}/10k) alleinTO={own:3d} p50={pct(th,.5)} p95={pct(th,.95)} p99={pct(th,.99)} p99.9={pct(th,.999)} >400:{sum(1 for l in th if 400<l<500)/max(1,len(th))*100:.2f}%  >430:{sum(1 for l in th if 430<l<500)/max(1,len(th))*100:.2f}%")
b=[m for g in G if g['start']<JOIN for m in g['moves']]; a=[m for g in G if g['start']>=JOIN for m in g['moves']]
row('vor Beitritt',b); row('nach Beitritt',a)
print('--- Nachher nach Parallelitaet')
for c in range(1,8):
    row(f'parallel={c}',[m for m in a if m['c']==c])
print('--- Vorher nach Parallelitaet')
for c in range(1,6):
    row(f'parallel={c}',[m for m in b if m['c']==c])
print('--- nach Modus (nachher)')
for mo in ['standard','duel','royale','constrictor']:
    row(mo,[m for g in G if g['mode']==mo and g['start']>=JOIN for m in g['moves']])
print('--- Spiele')
for mo in ['standard','duel','royale','constrictor']:
    gs=[g for g in G if g['mode']==mo and g['start']>=JOIN]
    pre=[g for g in G if g['mode']==mo and g['start']<JOIN]
    if pre: print(mo,'vor: n',len(pre),'win',round(sum(g['win'] for g in pre)/len(pre),3))
    if gs: print(mo,'nach: n',len(gs),'win',round(sum(g['win'] for g in gs)/len(gs),3),'avgplace',round(st.mean(g['place'] for g in gs),2),'med turns',st.median(g['turns'] for g in gs),'moves/spiel',round(st.mean(len(g['moves']) for g in gs)),'dauer min',round(st.mean(g['end']-g['start'] for g in gs)/60,1),'draws',0)
print('--- Timeouts: Folgen (nachher und vorher)')
n=0
for g in G:
    to=[m for m in g['moves'] if m['lat']>=500 or m['err']]
    for m in to:
        deadsoon = g['dcause'] and g['dturn'] is not None and 0<=g['dturn']-m['turn']<=2
        n+=1
        print(dt.datetime.fromtimestamp(m['t'],dt.timezone.utc).strftime('%m-%d %H:%M:%S'),g['mode'],'turn',m['turn'],'par',m['c'],'otherTO',m['otherto'],'tod' if deadsoon else '', g['dcause'] or '', g['dturn'] or '', 'place',g['place'],'WIN' if g['win'] else '')
