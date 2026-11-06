import json,sys,collections as C,statistics as st
root=sys.argv[1]
H=json.load(open(root+'/entries/hist.json'))['rows']
FIX='2026-11-05 10:50';ME='honey_badger'
pl=lambda s:1 if '1st' in s else 2 if '2nd' in s else 3 if '3rd' in s else 4 if '4th' in s else 0
OUT={}
for b in ['duel','standard','royale','constrictor']:
    res=[]
    for r in H[b]:
        if r['t']<FIX: continue
        fr=json.load(open(f"{root}/raw/{b}_{r['id']}.json"))['frames']
        if len(fr)<8: continue
        last=[s for s in fr[-1]['Snakes'] if s['Name']==ME][0]
        d=last.get('Death')
        if not d: continue
        T=d['Turn']
        lat={}
        ln={}
        for f in fr:
            for s in f['Snakes']:
                if s['Name']==ME and (not s.get('Death') or s['Death']['Turn']>=f['Turn']):
                    try: lat[f['Turn']]=int(s['Latency'] or 0)
                    except: lat[f['Turn']]=0
                    ln[f['Turn']]=len(s['Body'])
        run=0;t=T
        while t in lat and lat[t]<100 and t>0: run+=1;t-=1
        oth=[s for s in fr[-1]['Snakes'] if s['Name']!=ME]
        olen=0
        for f in fr[::-1]:
            if f['Turn']<=t:
                x=[len(s['Body']) for s in f['Snakes'] if s['Name']!=ME and not (s.get('Death') and s['Death']['Turn']<=f['Turn'])]
                olen=max(x) if x else 0;break
        res.append(dict(id=r['id'][:8],p=pl(r['pl']),T=T,c=d['Cause'],run=run,dec=t,lead=ln.get(t,0)-olen,ln=ln.get(t,0),opp=r['opp'][0]))
    OUT[b]=res
    rr=[x['run'] for x in res]
    print(b,'Tode',len(res),'erzwungene Serie vor Tod: median',st.median(rr),'>=5:',sum(v>=5 for v in rr),'>=10:',sum(v>=10 for v in rr),'>=20:',sum(v>=20 for v in rr))
    print('   nach Ursache',{k:(len(v),st.median(v)) for k,v in ((c,[x['run'] for x in res if x['c']==c]) for c in {x['c'] for x in res})})
    if b in('duel','standard'):
        print('   Vorsprung bei letzter Entscheidung: >=+3:',sum(x['lead']>=3 for x in res),'>=+10:',sum(x['lead']>=10 for x in res),'gleich(-2..2):',sum(-2<=x['lead']<=2 for x in res),'<=-3:',sum(x['lead']<=-3 for x in res))
for x in sorted(OUT['duel'],key=lambda x:-x['run'])[:30]: print(x)
json.dump(OUT,open(root+'/forced.json','w'),ensure_ascii=False)
