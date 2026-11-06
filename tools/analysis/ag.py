import csv,sys,json,collections as C,statistics as st
f,root=sys.argv[1],sys.argv[2]
R=list(csv.DictReader(open(f)))
for r in R:
    r['turn']=int(r['turn']);r['s']=float(r['v_ours']);r['won']=int(r['won']);r['len']=int(r['len']);r['maxopp']=int(r['maxopp'])
print('rows',len(R),'games',len({r['game'] for r in R}))
G=C.defaultdict(list)
for r in R: G[r['game']].append(r)
fin=[r['s'] for r in R if abs(r['s'])<9e5]
q=sorted(fin);print('score quantile',[q[int(len(q)*p)] for p in(.01,.1,.25,.5,.75,.9,.99)])
def auc(p):
    a=[x for x,y in p if y];b=[x for x,y in p if not y]
    if not a or not b:return None
    n=0
    # rank-basiert
    s=sorted(p,key=lambda t:t[0]);ra={};i=0
    while i<len(s):
        j=i
        while j<len(s) and s[j][0]==s[i][0]: j+=1
        for k in range(i,j): ra.setdefault(id(s[k]),(i+j+1)/2)
        i=j
    return round((sum(ra[id(t)] for t in s if t[1])-len(a)*(len(a)+1)/2)/(len(a)*len(b)),3)
print('AUC: Zugfenster -> Score, Laengenvorsprung')
for a,b in((10,50),(50,100),(100,150),(150,200),(200,300),(300,400),(400,1000)):
    p=[];l=[]
    for g,rs in G.items():
        x=[r for r in rs if a<=r['turn']<b]
        if x:
            p.append((st.median(r['s'] for r in x),x[0]['won']));l.append((st.mean(r['len']-r['maxopp'] for r in x),x[0]['won']))
    print((a,b),'n',len(p),'wins',sum(y for _,y in p),'AUC score',auc(p),'AUC len',auc(l))
print('--- verlorene Spiele: Sichtweite')
out=[]
for g,rs in G.items():
    rs.sort(key=lambda r:r['turn'])
    if rs[0]['won']: continue
    end=rs[-1]['turn']
    lossat=next((r['turn'] for r in rs if r['s']<-9e5),None)
    # Ab wann bleibt der Score dauerhaft schlecht (<−30)?
    neg=None
    for r in rs:
        if all(x['s']<-30 for x in rs if x['turn']>=r['turn']): neg=r['turn'];break
    pre=[r for r in rs if r['turn']<end-40]
    peak=max((r['s'] for r in pre),default=None)
    tail=[(r['turn'],round(r['s'])) for r in rs[-40::6]]
    out.append(dict(g=g[:8],end=end,lossat=lossat,neg=neg,pk=peak,lead=rs[-1]['len']-rs[-1]['maxopp'],tail=tail))
for o in sorted(out,key=lambda o:o['end']):print(o['g'],'Ende',o['end'],'Verlust gesehen -',(o['end']-o['lossat']) if o['lossat'] else None,'dauerhaft neg -',(o['end']-o['neg']) if o['neg'] else None,'peak40',round(o['pk'] or 0),'dLen',o['lead'],o['tail'][-5:])
v=[o['end']-o['lossat'] for o in out if o['lossat']]
print('Sicht bis Tod: median',st.median(v),'Viertel',sorted(v)[len(v)//4],sorted(v)[3*len(v)//4])
w=[o['end']-o['neg'] for o in out if o['neg']]
print('dauerhaft neg vor Tod: median',st.median(w),'Viertel',sorted(w)[len(w)//4],sorted(w)[3*len(w)//4])
print('Spiele mit Peak>30 noch 40 Zuege vorm Ende',sum(1 for o in out if (o['pk'] or 0)>30),'von',len(out))
print('--- gewonnene Spiele: Anteil mit zwischendurch Score<-30')
ws=[rs for rs in G.values() if rs[0]['won']]
print(sum(1 for rs in ws if any(r['s']<-30 for r in rs)),len(ws),'mit gesehenem Verlust',sum(1 for rs in ws if any(r['s']<-9e5 for r in rs)))
