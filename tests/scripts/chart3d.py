import math
W,H=1600,960
s=130; c=math.cos(math.pi/6); sn=0.5
ox,oy=800,420
quarters=["Q1","Q2","Q3","Q4"]
series=[("Pohjoinen","#ff8a3d"),("Keskinen","#3dc5ff"),("Etelä","#b47cff")]
data=[[12,15,19,24],[9,13,14,18],[7,9,12,16]]
hs=14
def P(i,j,h=0): return (ox+(i-j)*c*s, oy+(i+j)*sn*s-h*hs)
def pts(*a): return " ".join(f"{x:.1f},{y:.1f}" for x,y in a)
def shade(hexc,f):
    r,g,b=[int(hexc[k:k+2],16) for k in (1,3,5)]
    return "#%02x%02x%02x"%tuple(max(0,min(255,int(v*f))) for v in (r,g,b))
o=[f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="380 40 1060 900" width="1060" height="900"><defs>']
o.append('<radialGradient id="fl" cx="50%" cy="50%" r="60%"><stop offset="0" stop-color="#1e2a52"/><stop offset="1" stop-color="#0c1228"/></radialGradient>')
o.append('<filter id="bl"><feGaussianBlur stdDeviation="14"/></filter>')
for k,(n,col) in enumerate(series):
    o.append(f'<linearGradient id="t{k}" x1="0" y1="0" x2="1" y2="1"><stop offset="0" stop-color="{shade(col,1.35)}"/><stop offset="1" stop-color="{shade(col,1.05)}"/></linearGradient>')
    o.append(f'<linearGradient id="l{k}" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="{shade(col,0.85)}"/><stop offset="1" stop-color="{shade(col,0.45)}"/></linearGradient>')
    o.append(f'<linearGradient id="r{k}" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="{shade(col,0.6)}"/><stop offset="1" stop-color="{shade(col,0.3)}"/></linearGradient>')
o.append('</defs>')
ni,nj=4,3
# floor
o.append(f'<polygon points="{pts(P(-.2,-.2),P(ni+.2,-.2),P(ni+.2,nj+.2),P(-.2,nj+.2))}" fill="url(#fl)" stroke="#3a4a80" stroke-width="2"/>')
for i in range(ni+1):
    o.append(f'<line x1="{P(i,0)[0]:.1f}" y1="{P(i,0)[1]:.1f}" x2="{P(i,nj)[0]:.1f}" y2="{P(i,nj)[1]:.1f}" stroke="#2c3a6a" stroke-width="1.5"/>')
for j in range(nj+1):
    o.append(f'<line x1="{P(0,j)[0]:.1f}" y1="{P(0,j)[1]:.1f}" x2="{P(ni,j)[0]:.1f}" y2="{P(ni,j)[1]:.1f}" stroke="#2c3a6a" stroke-width="1.5"/>')
# back walls with gridlines
o.append(f'<polygon points="{pts(P(0,nj),P(0,0),P(0,0,25),P(0,nj,25))}" fill="#16204a" opacity="0.55"/>')
o.append(f'<polygon points="{pts(P(0,0),P(ni,0),P(ni,0,25),P(0,0,25))}" fill="#1a2654" opacity="0.55"/>')
for v in (5,10,15,20,25):
    a,b,cc=P(0,nj,v),P(0,0,v),P(ni,0,v)
    o.append(f'<polyline points="{pts(a,b,cc)}" fill="none" stroke="#2c3a6a" stroke-width="1" stroke-dasharray="6 6"/>')
    o.append(f'<text x="{a[0]-14:.1f}" y="{a[1]+6:.1f}" fill="#8fa0d0" font-size="22" font-family="sans-serif" text-anchor="end">{v}</text>')
m=0.2
labels=[]
cells=sorted([(i+j,i,j) for i in range(ni) for j in range(nj)])
for _,i,j in cells:
    v=data[j][i]
    a0,a1,b0,b1=i+m,i+1-m,j+m,j+1-m
    # shadow
    o.append(f'<polygon points="{pts(P(a0+.15,b0+.15),P(a1+.35,b0+.15),P(a1+.35,b1+.35),P(a0+.15,b1+.35))}" fill="#000" opacity="0.35" filter="url(#bl)"/>')
    o.append(f'<polygon points="{pts(P(a0,b1),P(a1,b1),P(a1,b1,v),P(a0,b1,v))}" fill="url(#l{j})"/>')
    o.append(f'<polygon points="{pts(P(a1,b0),P(a1,b1),P(a1,b1,v),P(a1,b0,v))}" fill="url(#r{j})"/>')
    o.append(f'<polygon points="{pts(P(a0,b0,v),P(a1,b0,v),P(a1,b1,v),P(a0,b1,v))}" fill="url(#t{j})" stroke="{shade(series[j][1],1.6)}" stroke-width="1.5"/>')
    x,y=P(i+.5,j+.5,v)
    labels.append(f'<text x="{x:.1f}" y="{y-14:.1f}" fill="#fff" font-size="26" font-weight="700" stroke="#0c1228" stroke-width="5" paint-order="stroke" font-family="sans-serif" text-anchor="middle">{v}</text>')
o.extend(labels)
for i,q in enumerate(quarters):
    x,y=P(i+.5,nj+.45)
    o.append(f'<text x="{x+10:.1f}" y="{y+28:.1f}" fill="#c8d4f5" font-size="28" font-family="sans-serif" text-anchor="middle">{q}</text>')
for j,(n,col) in enumerate(series):
    x,y=P(ni+.35,j+.5)
    o.append(f'<text x="{x+12:.1f}" y="{y+10:.1f}" fill="{shade(col,1.3)}" font-size="28" font-weight="600" font-family="sans-serif">{n}</text>')
o.append('</svg>')
open('/tmp/chart3d.svg','w').write("".join(o))
