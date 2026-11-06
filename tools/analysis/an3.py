import json,sys,glob,datetime as dt,collections as C,statistics as st,math
root=sys.argv[1]
G=json.load(open(root+'/processed.json'))
U=dt.timezone.utc
JOIN=dt.datetime(2026,11,5,11,20,tzinfo=U).timestamp()
FIX=dt.datetime(2026,11,5,10,50,tzinfo=U).timestamp()
LAST=dt.datetime(2026,11,5,20,0,tzinfo=U).timestamp()
G.sort(key=lambda g:g['start'])
# 1 Ticks
ticks=[];cur=[]
for g in G:
    if cur and g['start']-cur[-1]['start']>120: ticks.append(cur);cur=[]
    cur.append(g)
ticks.append(cur)
A=[t for t in ticks if t[0]['start']>=JOIN]
print('ticks nachher',len(A),'Groesse',sorted(C.Counter(len(t) for t in A).items()),'Abstand min',round(st.median((A[i+1][0]['start']-A[i][0]['start'])/60 for i in range(len(A)-1)),1))
print('Spielstarts Streuung in tick s',round(st.median(t[-1]['start']-t[0]['start'] for t in A),1))
print('--- vorab festgelegt: Tick-Groesse -> Ergebnis (nachher)')
for mo in ['standard','duel','royale','constrictor']:
    b=C.defaultdict(list)
    for t in A:
        for g in t:
            if g['mode']==mo: b[len(t)].append(g)
    print(mo,{k:(len(v),round(sum(x['win'] for x in v)/len(v),2),round(st.mean(x['place'] for x in v),2)) for k,v in sorted(b.items())})
print('--- Standard/Duell nachher: Royale im selben Tick ja/nein')
for mo in ['standard','duel']:
    b=C.defaultdict(list)
    for t in A:
        r=any(g['mode']=='royale' for g in t)
        for g in t:
            if g['mode']==mo: b[r].append(g)
    print(mo,{k:(len(v),round(sum(x['win'] for x in v)/len(v),2),round(st.mean(x['place'] for x in v),2)) for k,v in sorted(b.items())})
# 3 Tail
ms=[m for g in G if g['start']>=JOIN for m in g['moves']]
full=[m for m in ms if m['lat']>=300]
print('--- Tail nachher, Rechenzuege',len(full))
for lo,hi in [(380,385),(385,390),(390,400),(400,420),(420,440),(440,460),(460,480),(480,499),(499,99999)]:
    print(lo,hi,sum(1 for m in full if lo<=m['lat']<hi))
print('err ohne lat>=500',sum(1 for m in ms if m['err'] and m['lat']<500))
# 4
to=[(g,m) for g in G if g['start']>=JOIN for m in g['moves'] if m['lat']>=500 or m['err']]
tb=C.Counter();mb=C.Counter()
bk=lambda t:'0-49' if t<50 else '50-149' if t<150 else '150-299' if t<300 else '300-499' if t<500 else '500+'
for g in G:
    if g['start']<JOIN: continue
    for m in g['moves']: mb[bk(m['turn'])]+=1
for g,m in to: tb[bk(m['turn'])]+=1
print('TO/10k nach Zug',{k:(tb[k],round(tb[k]/mb[k]*1e4,1)) for k in mb})
f60=sum(1 for g in G if g['start']>=JOIN for m in g['moves'] if m['t']-g['start']<60)
print('Zuege in ersten 60s',round(f60/len(ms)*100,1),'% , TOs dort',round(sum(1 for g,m in to if m['t']-g['start']<60)/len(to)*100,1),'%')
# 5 cross-game
gt=sorted((m['t'],g['id']) for g,m in to)
pairs=sum(1 for i in range(len(gt)) for j in range(i+1,len(gt)) if gt[j][0]-gt[i][0]<5 and gt[j][1]!=gt[i][1])
same=sum(1 for i in range(len(gt)) for j in range(i+1,len(gt)) if gt[j][0]-gt[i][0]<5 and gt[j][1]==gt[i][1])
print('TO-Paare <5s: verschiedene Spiele',pairs,'gleiches Spiel',same)
others=[];base=[];allm=[(m['t'],g['id'],m['lat']) for g in G if g['start']>=JOIN for m in g['moves'] if m['lat']>=300]
import bisect
allm.sort();T=[x[0] for x in allm]
for g,m in to:
    if m['c']<2: continue
    i=bisect.bisect_left(T,m['t']-2);j=bisect.bisect_right(T,m['t']+2)
    o=[x[2] for x in allm[i:j] if x[1]!=g['id']]
    others+=o
print('Latenz anderer eigener Spiele +-2s um einen TO: n',len(others),'p50',st.median(others),'>=395:',round(sum(l>=395 for l in others)/len(others)*100,1),'%, >=500:',sum(l>=500 for l in others))
print('Baseline >=395:',round(sum(m['lat']>=395 for m in full)/len(full)*100,2),'%')
# 6 tot
print('--- Tod <=2 Zuege nach TO')
d=[(g,m) for g,m in to if g['dcause'] and g['dturn'] is not None and 0<=g['dturn']-m['turn']<=2]
for g,m in d: print(g['mode'],'zug',m['turn'],g['dcause'],'tod',g['dturn'],'platz',g['place'],'otherTO',m['otherto'],g['id'][:8])
# 2 Gegner-Vergleich
cnt=C.Counter();t=C.Counter();
for f in glob.glob(root+'/games/*.json'):
    d=json.load(open(f))
    if d['created']/1e6<JOIN: continue
    for a in d['frames'][1:]:
        for s in a['S']:
            if s['D'] and s['DT'] is not None and s['DT']<a['T']: continue
            try:l=int(s['L'] or 0)
            except:l=0
            cnt[s['N']]+=1
            t[s['N']]+= (l>=500 or s['E'])
print('--- TO/10k aller Schlangen in unseren Spielen (nachher, min 3000 Zuege)')
r=sorted(((t[k]/cnt[k]*1e4,k,t[k],cnt[k]) for k in cnt if cnt[k]>=3000))
for x in r: print(f'{x[1]:24s} {x[0]:6.1f}/10k  {x[2]:4d}/{x[3]}')
