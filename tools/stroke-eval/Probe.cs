using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.Drawing;
using System.Drawing.Imaging;
using System.Globalization;
using System.IO;
using System.Runtime.InteropServices;
using System.Threading;
using System.Windows.Forms;

public static class StrokeProbe {
    [DllImport("user32.dll", CharSet=CharSet.Unicode)] static extern IntPtr FindWindow(string cls, IntPtr title);
    [DllImport("user32.dll")] static extern IntPtr SendMessage(IntPtr h, uint m, IntPtr w, IntPtr l);
    [DllImport("user32.dll")] static extern bool PostMessage(IntPtr h, uint m, IntPtr w, IntPtr l);
    [DllImport("user32.dll")] static extern bool SetCursorPos(int x, int y);
    [DllImport("user32.dll")] static extern bool GetCursorPos(out POINT p);
    [DllImport("user32.dll")] static extern bool IsWindowVisible(IntPtr h);
    [DllImport("user32.dll")] static extern bool SetForegroundWindow(IntPtr h);
    [DllImport("user32.dll")] static extern IntPtr GetForegroundWindow();
    [DllImport("user32.dll")] static extern bool SetProcessDpiAwarenessContext(IntPtr c);
    [DllImport("user32.dll")] static extern uint GetDpiForWindow(IntPtr h);
    [DllImport("user32.dll")] static extern bool RedrawWindow(IntPtr h, IntPtr r, IntPtr region, uint flags);
    [DllImport("user32.dll")] static extern uint SendInput(uint n, INPUT[] inputs, int size);
    [StructLayout(LayoutKind.Sequential)] struct POINT { public int x,y; }
    [StructLayout(LayoutKind.Sequential)] struct INPUT { public uint type; public MOUSEINPUT mouse; }
    [StructLayout(LayoutKind.Sequential)] struct MOUSEINPUT { public int dx,dy; public uint data,flags,time; public UIntPtr extra; }
    [DllImport("user32.dll", SetLastError=true)] static extern IntPtr CreateSyntheticPointerDevice(uint type, uint max, uint feedback);
    [DllImport("user32.dll", SetLastError=true)] static extern bool InjectSyntheticPointerInput(IntPtr device, [In] POINTER_TYPE_INFO[] input, uint count);
    [DllImport("user32.dll")] static extern void DestroySyntheticPointerDevice(IntPtr device);
    [StructLayout(LayoutKind.Sequential)] struct POINTER_INFO {
        public uint type,id,frame,flags;
        public IntPtr source,target;
        public POINT pixel,himetric,pixelRaw,himetricRaw;
        public uint time,history;
        public int input;
        public uint keys;
        public ulong performance;
        public uint change;
    }
    [StructLayout(LayoutKind.Sequential)] struct PEN_INFO {
        public POINTER_INFO info;
        public uint flags,mask,pressure,rotation;
        public int tiltX,tiltY;
    }
    [StructLayout(LayoutKind.Explicit,Size=152)] struct POINTER_TYPE_INFO {
        [FieldOffset(0)] public uint type;
        [FieldOffset(8)] public PEN_INFO pen;
    }
    static IntPtr window;
    static int left,top,width,height;
    static readonly CultureInfo CI = CultureInfo.InvariantCulture;
    static void Msg(uint m, int value=0) { SendMessage(window,m,(IntPtr)value,IntPtr.Zero); }
    static void Move(int x,int y) { SetCursorPos(left+x,top+y); Msg(0x200); }
    static double Ms(long ticks) { return ticks * 1000.0 / Stopwatch.Frequency; }
    static void Pump(int ms) {
        var s=Stopwatch.StartNew();
        while(s.ElapsedMilliseconds<ms) { Application.DoEvents(); Thread.Sleep(2); }
    }
    static void Paint() { RedrawWindow(window,IntPtr.Zero,IntPtr.Zero,0x1|0x100|0x40); }
    static void Capture() {
        PostMessage(window,0x0312,(IntPtr)0x4D59,IntPtr.Zero);
        for(int i=0;i<300 && !IsWindowVisible(window);i++) Pump(10);
        if(!IsWindowVisible(window)) throw new Exception("Capture did not open");
        SendMessage(window,0,IntPtr.Zero,IntPtr.Zero);
        SetForegroundWindow(window); Pump(150);
        Move(0,0); Msg(0x204); Move(width,height); Msg(0x205); Paint();
    }
    static void Stroke(int index) {
        int span=width-120;
        int row=height/2+20+(index%12)*Math.Max(6,(height/2-70)/12);
        Move(60,row); Msg(0x201);
        for(int i=1;i<=80;i++) Move(60+span*i/80,row+(int)(12*Math.Sin(i*0.23+index)));
        Msg(0x202);
    }
    static void History(int n) {
        // Suppress display only during setup. Timed paints and input replay re-enable it.
        Msg(0xB,0);
        for(int i=0;i<n;i++) Stroke(i);
        Msg(0xB,1); Paint(); Pump(100);
    }
    static double Percentile(List<double> a,double p) {
        a.Sort(); return a[Math.Min(a.Count-1,(int)Math.Ceiling(p*a.Count)-1)];
    }
    static void Mouse(int x,int y,uint flags) {
        var screen=SystemInformation.VirtualScreen;
        var input=new INPUT { type=0, mouse=new MOUSEINPUT {
            dx=(int)Math.Round((left+x-screen.Left)*65535.0/(screen.Width-1)),
            dy=(int)Math.Round((top+y-screen.Top)*65535.0/(screen.Height-1)),
            flags=flags|0x8000|0x4000
        }};
        if(SendInput(1,new[]{input},Marshal.SizeOf(typeof(INPUT)))!=1) throw new Exception("SendInput failed");
    }
    static void RealCurve(string folder,int n,StreamWriter replay,int seconds) {
        int span=width-100;
        int center=height/4;
        int amplitude=Math.Min(60,height/7);
        int count=240*seconds;
        double duration=1000.0*seconds;
        double phase=Math.PI*10*seconds;
        Mouse(50,center,1); Pump(40); Mouse(50,center,2); Thread.Sleep(12);
        var clock=Stopwatch.StartNew();
        double worstLateness=0;
        for(int i=0;i<=count;i++) {
            double due=i*duration/count;
            while(clock.Elapsed.TotalMilliseconds<due) {
                double remain=due-clock.Elapsed.TotalMilliseconds;
                if(remain>1) Thread.Sleep(0); else Thread.SpinWait(20);
            }
            if(!IsWindowVisible(window) || GetForegroundWindow()!=window) throw new Exception("Replay lost its visible foreground window; run invalid");
            if(i>0) {
                POINT observed; GetCursorPos(out observed);
                int previousX=left+50+(int)Math.Round(span*(i-1)/(double)count);
                int previousY=top+center+(int)Math.Round(amplitude*Math.Sin((i-1)/(double)count*phase));
                if(Math.Abs(observed.x-previousX)>2 || Math.Abs(observed.y-previousY)>2) throw new Exception("External pointer movement during replay; run invalid");
            }
            int x=50+(int)Math.Round(span*i/(double)count);
            int y=center+(int)Math.Round(amplitude*Math.Sin(i/(double)count*phase));
            double actual=clock.Elapsed.TotalMilliseconds;
            worstLateness=Math.Max(worstLateness,actual-due);
            Mouse(x,y,1);
            replay.WriteLine(n+","+i+","+actual.ToString("F3",CI)+","+x+","+y);
        }
        if(!IsWindowVisible(window) || GetForegroundWindow()!=window) throw new Exception("Replay lost focus or capture; run invalid");
        Mouse(width-50,center,4);
        Pump(700); Paint(); Pump(100);
        using(var bitmap=new Bitmap(width,height,PixelFormat.Format32bppArgb)) {
            using(var g=Graphics.FromImage(bitmap)) g.CopyFromScreen(left,top,0,0,new Size(width,height));
            bitmap.Save(Path.Combine(folder,"curve-"+n+".png"),ImageFormat.Png);
            var distances=new List<double>();
            int missing=0;
            // Image-space centerline coverage. Thin endpoints are excluded.
            for(int i=10;i<count-10;i++) {
                int x=50+(int)Math.Round(span*i/(double)count);
                int y=center+(int)Math.Round(amplitude*Math.Sin(i/(double)count*phase));
                double nearest=21;
                for(int dy=-20;dy<=20;dy++) for(int dx=-20;dx<=20;dx++) {
                    double d=Math.Sqrt(dx*dx+dy*dy);
                    if(d>=nearest || x+dx<0 || x+dx>=width || y+dy<0 || y+dy>=height) continue;
                    var color=bitmap.GetPixel(x+dx,y+dy);
                    if(color.R>180 && color.G<150 && color.B<160) nearest=d;
                }
                distances.Add(nearest); if(nearest>3) missing++;
            }
            File.AppendAllText(Path.Combine(folder,"fidelity.csv"),
                n+","+Percentile(distances,.5).ToString("F3",CI)+","+Percentile(distances,.95).ToString("F3",CI)+","+missing+","+distances.Count+","+worstLateness.ToString("F3",CI)+Environment.NewLine);
        }
    }
    static void HistoryCosts(string folder) {
        History(250);
        using(var output=new StreamWriter(Path.Combine(folder,"history-costs.csv"))) {
            output.WriteLine("action,dispatch_ms,paint_ms");
            foreach(int key in new[]{0x5A,0x59,0x52,0x5A,0x0D}) {
                long start=Stopwatch.GetTimestamp(); Msg(0x100,key); long input=Stopwatch.GetTimestamp();
                if(IsWindowVisible(window)) Paint();
                output.WriteLine(key+","+Ms(input-start).ToString("F3",CI)+","+Ms(Stopwatch.GetTimestamp()-input).ToString("F3",CI));
            }
        }
    }
    static void Shapes(string folder) {
        Action<int,int> begin = (x,y) => { Move(x,y); Msg(0x201); };
        begin(90,90); Move(240,90); Move(240,180); Move(390,180); Msg(0x202);
        begin(90,260); Move(500,260); Msg(0x202); // Two-event flick.
        for(int j=0;j<5;j++) { begin(90+j*50,340); Msg(0x202); Pump(270); }
        begin(700,120);
        for(int i=1;i<=240;i++) Move(700+(int)(100*Math.Sin(i*.08)),120+(int)(65*Math.Sin(i*.16)));
        Msg(0x202);
        begin(950,300); Move(1100,410); Move(950,410); Move(1100,300); Msg(0x202);
        // Space uses the same stroke path and width code.
        Move(90,430); Msg(0x100,32);
        for(int i=1;i<=60;i++) Move(90+i*6,430+(int)(22*Math.Sin(i*.2)));
        Msg(0x101,32);
        Paint(); Pump(100);
        using(var bitmap=new Bitmap(width,height,PixelFormat.Format32bppArgb)) {
            using(var g=Graphics.FromImage(bitmap)) g.CopyFromScreen(left,top,0,0,new Size(width,height));
            bitmap.Save(Path.Combine(folder,"shapes.png"),ImageFormat.Png);
        }
        var device=CreateSyntheticPointerDevice(3,1,3);
        if(device==IntPtr.Zero) throw new Exception("Synthetic pen unavailable: "+Marshal.GetLastWin32Error());
        try {
            for(int row=0;row<3;row++) {
                for(int i=0;i<=100;i++) {
                    uint pressure=(uint)(row==0?100:row==1?900:100+i*8);
                    var pen=new POINTER_TYPE_INFO { type=3, pen=new PEN_INFO {
                        info=new POINTER_INFO { type=3,id=1,flags=(uint)(i==0?0x10000|2|4:i==100?0x40000|2:0x20000|2|4),
                            pixel=new POINT { x=left+100+i*9,y=top+540+row*70 } }, mask=1,pressure=pressure
                    }};
                    if(!InjectSyntheticPointerInput(device,new[]{pen},1)) throw new Exception("Pen injection failed: "+Marshal.GetLastWin32Error());
                    Pump(5);
                }
                Pump(300);
            }
        } finally { DestroySyntheticPointerDevice(device); }
        Paint(); Pump(100);
        using(var bitmap=new Bitmap(width,height,PixelFormat.Format32bppArgb)) {
            using(var g=Graphics.FromImage(bitmap)) g.CopyFromScreen(left,top,0,0,new Size(width,height));
            bitmap.Save(Path.Combine(folder,"pressure.png"),ImageFormat.Png);
            var widths=new int[3];
            for(int row=0;row<3;row++) for(int y=520+row*70;y<560+row*70;y++) {
                var c=bitmap.GetPixel(550,y); if(c.R>180 && c.G<150 && c.B<160) widths[row]++;
            }
            File.WriteAllText(Path.Combine(folder,"pressure-widths.txt"),string.Join(",",widths));
            if(widths[0]<1 || widths[1]<=widths[0]) throw new Exception("Synthetic pressure did not change ink width: "+string.Join(",",widths));
            long start=Stopwatch.GetTimestamp(); Msg(0x100,13);
            double copy=Ms(Stopwatch.GetTimestamp()-start);
            using(var exported=(Bitmap)Clipboard.GetImage()) {
                if(exported==null || exported.Width!=width || exported.Height!=height) throw new Exception("Export size mismatch");
                exported.Save(Path.Combine(folder,"export.png"),ImageFormat.Png);
                int different=0,maskDifference=0,missing=0;
                Func<Color,bool> ink = c => c.R>180 && c.G<150 && c.B<160;
                for(int y=10;y<height-10;y++) for(int x=10;x<width-10;x++) {
                    var a=bitmap.GetPixel(x,y); var b=exported.GetPixel(x,y);
                    if(a.ToArgb()!=b.ToArgb()) different++;
                    if(ink(a)!=ink(b)) {
                        maskDifference++;
                        bool nearby=false;
                        for(int dy=-1;dy<=1 && !nearby;dy++) for(int dx=-1;dx<=1;dx++) {
                            if(ink(a)?ink(exported.GetPixel(x+dx,y+dy)):ink(bitmap.GetPixel(x+dx,y+dy))) { nearby=true; break; }
                        }
                        if(!nearby) missing++;
                    }
                }
                File.WriteAllText(Path.Combine(folder,"export-comparison.txt"),"copy_ms="+copy.ToString("F3",CI)+"\nexact_pixel_differences="+different+"\nmask_differences="+maskDifference+"\nmask_differences_beyond_1px="+missing+"\n");
                if(missing>0) throw new Exception("Preview/export ink differs beyond antialias edges: "+missing);
            }
        }
        Console.WriteLine("Shapes, synthetic pressure, and preview/export comparison passed");
    }
    static void LongStroke(string folder) {
        Action<int> point = i => Move(width/2+(int)(width*.3*Math.Sin(i*.07)),height/2+(int)(height*.3*Math.Sin(i*.09)));
        point(0); Msg(0x201);
        using(var output=new StreamWriter(Path.Combine(folder,"active.csv"))) {
            output.WriteLine("samples,frame,input_ms,paint_ms");
            int previous=0;
            foreach(int limit in new[]{80,320,640,5000,20000}) {
                Msg(0xB,0);
                for(int i=previous+1;i<=limit;i++) point(i);
                Msg(0xB,1);
                for(int frame=0;frame<6;frame++) {
                    var start=Stopwatch.GetTimestamp(); point(limit+frame+1); var input=Stopwatch.GetTimestamp(); Paint(); var end=Stopwatch.GetTimestamp();
                    output.WriteLine((limit+frame+1)+","+frame+","+Ms(input-start).ToString("F3",CI)+","+Ms(end-input).ToString("F3",CI));
                }
                output.Flush(); previous=limit+6;
                Console.WriteLine("active samples="+limit+" completed");
            }
        }
        Msg(0x202);
    }
    public static void Run(string exe,string folder,int[] densities,int frames,int seconds) {
        if(FindWindow("Myosotis.Native.Overlay",IntPtr.Zero)!=IntPtr.Zero) throw new Exception("Quit the existing Myosotis before diagnostics; no process was changed.");
        SetProcessDpiAwarenessContext((IntPtr)(-4));
        POINT pointer; GetCursorPos(out pointer); var previous=GetForegroundWindow();
        Process process=null;
        var fixture=new Form { Text="Myosotis performance test — white fixture", FormBorderStyle=FormBorderStyle.None,
            Bounds=Screen.PrimaryScreen.Bounds, BackColor=Color.White, ShowInTaskbar=false };
        try {
            fixture.Show(); fixture.Activate(); Pump(200);
            var screen=Screen.PrimaryScreen.Bounds;
            left=screen.Left+80; top=screen.Top+80;
            width=Math.Min(1400,screen.Width-160); height=Math.Min(850,screen.Height-220);
            if(width<700 || height<400) throw new Exception("Display too small for the diagnostic canvas");
            process=Process.Start(new ProcessStartInfo(exe){UseShellExecute=false});
            for(int i=0;i<300;i++) { window=FindWindow("Myosotis.Native.Overlay",IntPtr.Zero); if(window!=IntPtr.Zero) break; Pump(10); }
            if(window==IntPtr.Zero || process.HasExited) throw new Exception("Test app failed to start");
            File.WriteAllText(Path.Combine(folder,"environment.txt"),"exe="+exe+"\ncanvas="+width+"x"+height+"\nscreen="+screen+"\ndpi="+GetDpiForWindow(window)+"\nclock_hz="+Stopwatch.Frequency+"\n");
            File.WriteAllText(Path.Combine(folder,"fidelity.csv"),"history_strokes,median_distance_px,p95_distance_px,missing_over_3px,samples,max_inject_lateness_ms\n");
            using(var output=new StreamWriter(Path.Combine(folder,"frames.csv")))
            using(var replay=new StreamWriter(Path.Combine(folder,"input.csv"))) {
                output.WriteLine("history_strokes,frame,input_ms,paint_ms,private_bytes");
                replay.WriteLine("history_strokes,sample,time_ms,x,y");
                foreach(int n in densities) {
                    if(n==-4) { Capture(); HistoryCosts(folder); Msg(0x100,27); Pump(100); continue; }
                    if(n==-2) { Capture(); Shapes(folder); Msg(0x100,27); Pump(100); continue; }
                    if(n<0) { Capture(); LongStroke(folder); Msg(0x100,27); Pump(100); continue; }
                    Capture(); History(n);
                    using(var idle=new StreamWriter(Path.Combine(folder,"static-"+n+".csv"))) {
                        idle.WriteLine("frame,paint_ms");
                        for(int frame=0;frame<8;frame++) {
                            var start=Stopwatch.GetTimestamp(); Paint();
                            idle.WriteLine(frame+","+Ms(Stopwatch.GetTimestamp()-start).ToString("F3",CI));
                        }
                    }
                    Move(50,height/4); Msg(0x201);
                    for(int i=0;i<frames+5;i++) {
                        int x=70+i*(width-140)/(frames+5);
                        int y=height/4+(int)(30*Math.Sin(i*.3));
                        var t=Stopwatch.GetTimestamp(); Move(x,y); var input=Stopwatch.GetTimestamp(); Paint(); var end=Stopwatch.GetTimestamp();
                        if(i>=5) { process.Refresh(); output.WriteLine(n+","+(i-5)+","+Ms(input-t).ToString("F3",CI)+","+Ms(end-input).ToString("F3",CI)+","+process.PrivateMemorySize64); }
                    }
                    Msg(0x202); Msg(0x100,0x5A); // Remove the measurement stroke before fidelity replay.
                    RealCurve(folder,n,replay,seconds);
                    output.Flush(); replay.Flush();
                    Msg(0x100,27); Pump(100);
                    Console.WriteLine(Path.GetFileName(Path.GetDirectoryName(exe))+" history="+n+" completed");
                }
            }
        } finally {
            if(process!=null && !process.HasExited) {
                if(IsWindowVisible(window)) Msg(0x100,27);
                Msg(0x10); if(!process.WaitForExit(3000)) process.Kill();
            }
            fixture.Close(); fixture.Dispose();
            SetCursorPos(pointer.x,pointer.y); if(previous!=IntPtr.Zero) SetForegroundWindow(previous);
        }
    }
}
