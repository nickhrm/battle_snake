import re,glob,json,sys,collections as C,statistics as st,datetime as dt
d=sys.argv[1]
R={}
now={}
for b in ['standard','duel','royale','constrictor']:
    rows={}
    for f in glob.glob(f'{d}/{b}.*.html'):
        h=open(f).read()
        if b not in now:
            m=re.search(r'mu ([0-9.]+)[^g]{0,20}sigma ([0-9.]+)',h)
            if f.endswith('.1.html') and m: now[b]=(float(m[1]),float(m[2]))
        i=h.find('Game History')
        for tr in re.findall(r'<tr>(.*?)</tr>',h[i:],re.S):
            g=re.search(r'/games/([a-f0-9-]{36})',tr)
            if not g: continue
            ds=re.search(r'<td>(\d{4}-\d\d-\d\d \d\d:\d\d)</td>',tr)
            opp=re.findall(r'/entries/[a-f0-9-]+">([^<]+)</a>',tr)
            pl=re.search(r"class=.badge[^>]*>([^<]*)<",tr)
            de=re.search(r'rating-(?:positive|negative)[^>]*>([+-]?[0-9.]+)<',tr)
            if not de: de=re.search(r'class="r num">(?:<span[^>]*>)?([+-]?[0-9.]+)',tr)
            rows[g[1]]=dict(id=g[1],t=ds[1],opp=[o.strip() for o in opp],pl=pl[1] if pl else '',d=float(de[1]) if de else 0.0)
    R[b]=sorted(rows.values(),key=lambda r:(r['t'],r['id']))
json.dump({'rows':R,'now':now},open(d+'/hist.json','w'),ensure_ascii=False)
EP=[('Basis bis 05.11. 10:50','0000','2026-11-05 10:50'),('4d079b7..36f0069','2026-11-05 10:50','2026-11-05 20:30'),('70e89ce (Royale-Profil)','2026-11-05 20:30','9999')]
print('now',now)
for b,rs in R.items():
    tot=sum(r['d'] for r in rs); cur=now[b][0]-3*now[b][1]
    print(f'\n{b}: n={len(rs)} aktuell {cur:.1f} Summe Deltas {tot:.1f} (Start {cur-tot:.1f})')
    for n,a,c in EP:
        x=[r for r in rs if a<=r['t']<c]
        if x: print(f'  {n:26s} n={len(x):3d} d/Spiel {sum(r["d"] for r in x)/len(x):+.3f} Summe {sum(r["d"] for r in x):+.1f}')
    # Fenster letzte k Spiele
    for k in (25,50,100):
        if len(rs)>=k: print(f'  letzte {k}: {sum(r["d"] for r in rs[-k:]):+.2f}')
    # 12h Bloecke
    b12=C.OrderedDict()
    for r in rs:
        k=r['t'][5:10]+(' am' if int(r['t'][11:13])<12 else ' pm')
        b12.setdefault(k,[]).append(r['d'])
    print('  ',{k:(len(v),round(sum(v),2)) for k,v in b12.items()})
