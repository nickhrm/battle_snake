import glob,re,subprocess,sys,statistics as st
bin_,suite=sys.argv[1:3];modes=sys.argv[3:] or ['seq','par']
for b in ('duel','standard'):
    for m in modes:
        D=[];N=[];sk=0
        for f in sorted(glob.glob(f'{suite}/{b}_*.json')):
            o=subprocess.run([bin_,f,'300',m],capture_output=True,text=True).stdout.strip().split('\n')[-1]
            r=re.search(r'depth=(\d+) score=(\S+) nodes=(\d+)',o)
            if not r or int(r[1])==0 or abs(float(r[2]))>9e5: sk+=1;continue
            D.append(int(r[1]));N.append(int(r[3]))
        print(b,m,'n',len(D),'skip',sk,'Tiefe mean',round(st.mean(D),2),'median',st.median(D),'knoten',round(st.mean(N)/1000),'k')
