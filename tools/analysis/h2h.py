import json,re,sys,glob,collections as C,statistics as st,math
d=sys.argv[1]
H=json.load(open(d+'/entries/hist.json')); R=H['rows']
FIX='2026-11-05 10:50'
# Ranglisten
rank={}
names={'Constrictor':'2997e28a-83a0-41e0-a203-d42108cfe450','Duels':'77212f22-20f5-4e0c-8db6-78844318a9bc','Royale':'77ce41bf-12ad-4eac-bac2-b11939fb9b18','Standard':'d692fd01-5376-4060-8c4c-91dc2ecfe3ba'}
key={'Constrictor':'constrictor','Duels':'duel','Royale':'royale','Standard':'standard'}
for k,i in names.items():
    t=re.sub(r'<script.*?</script>|<style.*?</style>','',open(f'{d}/lb_{i}.html').read(),flags=re.S)
    rk={}
    for m in re.finditer(r'/entries/[a-f0-9-]+"[^>]*>(?:<[^>]+>)*([^<]+)<',t):
        n=m[1].strip()
        if n and n not in rk: rk[n]=len(rk)+1
    rank[key[k]]=rk
def ci(x):
    m=st.mean(x); s=st.stdev(x)/math.sqrt(len(x)); return m,s
print('--- Trend seit Fix (rating-Delta pro Spiel, 95%CI), hochgerechnet /100 Spiele')
for b,rs in R.items():
    x=[r['d'] for r in rs if r['t']>=FIX]
    late=[r['d'] for r in rs if r['t']>='2026-11-05 14:00']
    m,s=ci(late); print(b,'seit 14:00: n',len(late),f'{m*100:+.1f}/100 Spiele CI {(m-2*s)*100:+.1f}..{(m+2*s)*100:+.1f}')
pl=lambda s:1 if '1st' in s else 2 if '2nd' in s else 3 if '3rd' in s else 4 if '4th' in s else None
print('--- Duel seit Fix pro Gegner')
rs=[r for r in R['duel'] if r['t']>=FIX]
c=C.defaultdict(lambda:[0,0,0])
for r in rs:
    o=r['opp'][0]; p=pl(r['pl'])
    c[o][0 if p==1 else 1 if p==2 else 2]+=1
tot=lambda rmin,rmax:[sum(v[i] for o,v in c.items() if rmin<=rank['duel'].get(o,99)<=rmax) for i in range(3)]
for o,v in sorted(c.items(),key=lambda kv:rank['duel'].get(kv[0],99)):
    print(f"#{rank['duel'].get(o,'?')} {o}: {v[0]}-{v[1]}-{v[2]}")
for a,b_ in [(1,5),(6,15),(16,99)]: print('Rang',a,b_,tot(a,b_))
print('--- 4er seit Fix: Platz nach Gegnerstaerke (bester Rang im Feld)')
for b in ['standard','royale','constrictor']:
    rs=[r for r in R[b] if r['t']>=FIX]
    g=C.defaultdict(list)
    for r in rs:
        rk=[rank[b].get(o,99) for o in r['opp']]
        avg=st.mean(rk)
        g['A avg<=6' if avg<=6 else 'B avg<=11' if avg<=11 else 'C rest'].append(r)
    print(b,len(rs),{k:(len(v),'win',round(sum(pl(x['pl'])==1 for x in v)/len(v),2),'place',round(st.mean(pl(x['pl']) or 2 for x in v),2),'d/Spiel',round(st.mean(x['d'] for x in v),2)) for k,v in sorted(g.items())})
    # pro Gegner
    o=C.defaultdict(list)
    for r in rs:
        for n in r['opp']: o[n].append(r)
    li=[(rank[b].get(n,99),n,len(v),round(sum(pl(x['pl'])==1 for x in v)/len(v),2),round(st.mean(pl(x['pl']) or 2 for x in v),2),round(st.mean(x['d'] for x in v),2)) for n,v in o.items() if len(v)>=8]
    print(' Gegner(n>=8): rang,name,n,win,place,d')
    for l in sorted(li): print('   ',l)
