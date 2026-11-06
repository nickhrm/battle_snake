import json,re,sys,collections as C,statistics as st,math
root=sys.argv[1]
H=json.load(open(root+'/entries/hist.json'))['rows']['royale']
t=re.sub(r'<script.*?</script>|<style.*?</style>','',open(root+'/lb_77ce41bf-12ad-4eac-bac2-b11939fb9b18.html').read(),flags=re.S)
rk={}
for m in re.finditer(r'/entries/[a-f0-9-]+"[^>]*>(?:<[^>]+>)*([^<]+)<',t):
    n=m[1].strip()
    if n and n not in rk: rk[n]=len(rk)+1
pl=lambda s:1 if '1st' in s else 2 if '2nd' in s else 3 if '3rd' in s else 4 if '4th' in s else 2.5
P='2026-11-05 20:30'
H=[r for r in H]
# chronologische Fenster
print('Fenster: n, oppRang, win, place, hazard/hunger Tode')
def death(r):
    d=json.load(open(f"{root}/raw/royale_{r['id']}.json"))
    s=[x for x in d['frames'][-1]['Snakes'] if x['Name']=='honey_badger'][0].get('Death')
    return d and (s['Cause'] if s else 'lebt')
for r in H: r['dc']=death(r); r['ork']=st.mean(rk.get(o,30) for o in r['opp'])
for name,sel in (('vor Profil',[r for r in H if r['t']<P]),('Profil',[r for r in H if r['t']>=P])):
    print(name,len(sel),'oppRang',round(st.mean(r['ork'] for r in sel),1),'win',round(sum(pl(r['pl'])==1 for r in sel)/len(sel),3),'place',round(st.mean(pl(r['pl']) for r in sel),2),'hazard',sum(r['dc']=='hazard' for r in sel),'hunger',sum(r['dc']=='out-of-health' for r in sel))
print('--- gleicher Gegner-Staerke-Korb: mittl. Gegnerrang')
for a,b in ((0,13),(13,17),(17,40)):
    for name,sel in (('vor',[r for r in H if r['t']<P]),('Profil',[r for r in H if r['t']>=P])):
        s=[r for r in sel if a<=r['ork']<b]
        if s: print(f'Rang {a}-{b}',name,len(s),'win',round(sum(pl(r['pl'])==1 for r in s)/len(s),2),'place',round(st.mean(pl(r['pl']) for r in s),2),'hazard',sum(r['dc']=='hazard' for r in s))
# Fisher hazard
from math import comb
a,b,c,d=8,34,21,31
tot=sum(comb(a+b,k)*comb(c+d,a+c-k) for k in range(0,a+1))/comb(a+b+c+d,a+c)
print('Fisher einseitig hazard',round(tot,4))
# nach letzte Fenster
s=[r for r in H if r['t']>=P]
for i in range(0,len(s),13):
    x=s[i:i+13];print(x[0]['t'][5:13],len(x),'Rating',round(sum(y['d'] for y in x),2))
