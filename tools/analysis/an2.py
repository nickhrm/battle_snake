import json,sys,datetime as dt,collections as C,statistics as st
G=json.load(open(sys.argv[1]+'/processed.json'))
U=dt.timezone.utc
h=C.Counter();n=C.Counter()
for g in G:
    for m in g['moves']:
        k=dt.datetime.fromtimestamp(m['t'],U).strftime('%m-%d %H')
        n[k]+=1;h[k]+= 430<m['lat']<500
last=[k for k in sorted(n) if h[k]/n[k]>.02]
print('letzte Stunde mit >430ms:',last[-1:] )
JOIN=dt.datetime(2026,11,5,11,20,tzinfo=U).timestamp()
A=[g for g in G if g['start']>=JOIN]
full=sum(sum(1 for m in g['moves'] if m['lat']>=300) for g in A)
print('Last-Anteil nach Modus (Zuege >=300ms = volle Rechenzeit):')
for mo in ['standard','duel','royale','constrictor']:
    gs=[g for g in A if g['mode']==mo]
    f=sum(sum(1 for m in g['moves'] if m['lat']>=300) for g in gs)
    print(mo,len(gs),'spiele; rechenzuege',f,f'{f/full*100:.1f}%')
print('span h',(max(g['end'] for g in A)-JOIN)/3600)
def occ(gs):
    ev=[]
    for g in gs: ev+=[(g['start'],1),(g['end'],-1)]
    ev.sort();c=0;p=ev[0][0];T=C.Counter()
    for t,d in ev:
        T[c]+=t-p;p=t;c+=d
    return {k:round(v/60) for k,v in sorted(T.items())}
print('Minuten mit k Spielen',occ(A))
print('ohne R/C',occ([g for g in A if g['mode'] in('standard','duel')]))
print('ohne R',occ([g for g in A if g['mode']!='royale']))
def rate(ms):
    return sum(1 for m in ms if m['lat']>=500 or m['err']),len(ms)
print('Standard+Duell-Zuege nach Anzahl paralleler Royale/Constrictor')
gs=[g for g in A if g['mode'] in('standard','duel')]
R=[g for g in A if g['mode'] in('royale','constrictor')]
b=C.defaultdict(list)
for g in gs:
    for m in g['moves']:
        b[sum(1 for h in R if h['start']<=m['t']<=h['end'])].append(m)
for k in sorted(b):
    ms=b[k];th=sorted(m['lat'] for m in ms if m['lat']>=300)
    print(k,rate(ms),f"{rate(ms)[0]/len(ms)*1e4:.1f}/10k",'p99',th[int(len(th)*.99)] if th else 0,'>=395:',round(sum(l>=395 for l in th)/max(1,len(th))*100,3),'%')
print('Ergebnis nach mittlerer Parallelitaet: (n,win,place)')
for mo in ['standard','duel','royale','constrictor']:
    b=C.defaultdict(list)
    for g in [x for x in A if x['mode']==mo]:
        if not g['moves']: continue
        c=st.mean(m['c'] for m in g['moves']); b['1:<=2' if c<=2 else ('2:2-3' if c<=3 else '3:>3')].append(g)
    print(mo,{k:(len(v),round(sum(x['win'] for x in v)/len(v),2),round(st.mean(x['place'] for x in v),2)) for k,v in sorted(b.items())})
to=[(g,m) for g in A for m in g['moves'] if m['lat']>=500 or m['err']]
print('TO nachher',len(to),'allein',sum(1 for g,m in to if m['otherto']==0),'in nicht gewonnenen Spielen',sum(1 for g,m in to if not g['win']),'Tod <=2 Zuege',sum(1 for g,m in to if g['dcause'] and g['dturn'] is not None and 0<=g['dturn']-m['turn']<=2))
print('TO pro Mode',C.Counter(g['mode'] for g,m in to))
print('TO Stunde UTC',sorted(C.Counter(dt.datetime.fromtimestamp(m['t'],U).strftime('%H') for g,m in to).items()))
print('TO Zug<10',sum(1 for g,m in to if m['turn']<10),'erste 60s',sum(1 for g,m in to if m['t']-g['start']<60))
ts=sorted(m['t'] for g,m in to); print('TO in <5s Naehe eines anderen TO',sum(1 for i,t in enumerate(ts) if (i and t-ts[i-1]<5) or (i+1<len(ts) and ts[i+1]-t<5)))
