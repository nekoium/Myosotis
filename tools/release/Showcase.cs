using System;
using System.Drawing;
using System.Drawing.Drawing2D;
using System.Drawing.Imaging;
using System.Drawing.Text;
using System.Globalization;
using System.IO;
using System.Windows.Forms.DataVisualization.Charting;

public static class ReleaseShowcase {
    static Graphics g;
    static string root;
    const string Ink="#202634", Muted="#647084", Paper="#F3F4F7", Coral="#C74656", Accent="#5865D8";
    static Color C(string value) { return ColorTranslator.FromHtml(value); }
    static void Fill(string color,float x,float y,float w,float h) { using(var b=new SolidBrush(C(color))) g.FillRectangle(b,x,y,w,h); }
    static void Text(string value,float x,float y,float size=22,string color=Ink,bool bold=false) {
        using(var f=new Font("Segoe UI",size,bold?FontStyle.Bold:FontStyle.Regular,GraphicsUnit.Pixel))
        using(var b=new SolidBrush(C(color))) {
            var s=g.MeasureString(value,f,4000,StringFormat.GenericTypographic);
            if(x+s.Width>1780 || x<0) throw new Exception("Text exceeds canvas: "+value);
            g.DrawString(value,f,b,new PointF(x,y),StringFormat.GenericTypographic);
        }
    }
    static string P(string value) { return Path.Combine(root,value.Replace('/',Path.DirectorySeparatorChar)); }
    static void Crop(string file,float sx,float sy,float sw,float sh,float x,float y,float w,float h) {
        using(var image=Image.FromFile(P(file))) {
            if(sx<0 || sy<0 || sx+sw>image.Width || sy+sh>image.Height) throw new Exception("Invalid crop: "+file);
            g.DrawImage(image,new RectangleF(x,y,w,h),new RectangleF(sx,sy,sw,sh),GraphicsUnit.Pixel);
        }
    }
    static void Header(string title,string subtitle,string index) {
        g.Clear(C(Paper));
        Text("MYOSOTIS  /  0.2.0",64,42,22,Accent,true);
        Text(index,1580,42,20,Muted);
        Text(title,64,91,48,Ink,true);
        Text(subtitle,66,158,23,Muted);
    }
    static void Save(string name,int height,Action draw) {
        using(var bitmap=new Bitmap(1800,height)) {
            bitmap.SetResolution(144,144);
            using(g=Graphics.FromImage(bitmap)) {
                g.SmoothingMode=SmoothingMode.AntiAlias;
                g.InterpolationMode=InterpolationMode.HighQualityBicubic;
                g.PixelOffsetMode=PixelOffsetMode.HighQuality;
                g.TextRenderingHint=TextRenderingHint.AntiAliasGridFit;
                draw();
            }
            bitmap.Save(P("docs/releases/0.2.0/"+name),ImageFormat.Png);
        }
    }
    const string Before=".tmp/release-0.2.0/captures/before/scene.png";
    const string After=".tmp/release-0.2.0/captures/after-dark/scene.png";
    static void Badges(string file,float y) {
        int[] cx={864,1268,1268}, cy={396,474,630};
        for(int i=0;i<3;i++) Crop(file,cx[i]-44,cy[i]-34,88,68,1040+i*210,y,132,102);
    }
    static void BeforeAfter() {
        Save("01-before-after.png",1400,()=>{
            Header("A more natural mark.","Refined coral ink, tapered strokes, and numbers with room to breathe.","01 / 03");
            Text("BEFORE  /  0.1.2",64,224,24,Muted,true);
            Text("AFTER  /  0.2.0",916,224,24,Accent,true);
            Crop(Before,62,98,1354,866,64,272,820,524);
            Crop(After,62,98,1354,866,916,272,820,524);
            Text("Original coral + blue controls",64,811,22,Muted);
            Text("Refined coral + periwinkle controls",916,811,22,Muted);
            Fill("#DDE1E9",64,866,1672,1);
            Text("Strokes  /  enlarged detail",64,902,28,Ink,true);
            Text("Numbered marks  /  enlarged detail",916,902,28,Ink,true);
            Text("0.1.2",64,980,22,Muted); Text("0.2.0",64,1130,22,Accent,true);
            Crop(Before,133,326,506,54,172,969,708,76);
            Crop(After,133,326,506,54,172,1119,708,76);
            Text("0.1.2",916,1010,22,Muted);Badges(Before,968);
            Text("0.2.0",916,1160,22,Accent,true);Badges(After,1118);
            Text("Same captured path. Thinner starts and ends.",64,1240,22,Muted);
            Text("Centered numerals. Badges widen for multiple digits.",916,1240,22,Muted);
            Text("Native app captures on the same synthetic scene at 200% display scaling. Details enlarged equally.",64,1330,19,Muted);
        });
    }
    static void Components() {
        Save("02-components.png",1600,()=>{
            Header("Small controls. Clear feedback.","Both interface themes, four drawing tools, and contextual guidance.","02 / 03");
            Text("DARK",64,226,22,Muted,true);Text("LIGHT",650,226,22,Muted,true);
            string[] labels={"Pen  /  1","Arrow  /  2","Rectangle  /  3","Numbered mark  /  4"};
            for(int i=0;i<4;i++) {
                float y=276+i*116;
                Crop(".tmp/release-0.2.0/captures/after-dark/tool-"+(i+1)+".png",64,794,480,88,64,y,480,88);
                Crop(".tmp/release-0.2.0/captures/after-light/tool-"+(i+1)+".png",64,794,480,88,650,y,480,88);
                Text(labels[i],1230,y+28,26,Ink,true);
            }
            Text("Guidance when you need it",64,775,28,Ink,true);
            Crop(After,64,898,798,64,64,832,798,64);
            Crop(".tmp/release-0.2.0/captures/after-light/hover-pen.png",64,898,468,64,974,832,468,64);
            Text("Copy or cancel without a keyboard reach.",64,913,21,Muted);
            Text("Hover for the tool name and shortcut.",974,913,21,Muted);
            Fill("#DDE1E9",64,972,1672,1);
            Text("Coral marks. Familiar tools.",64,1008,28,Ink,true);
            Crop(After,133,326,506,54,64,1076,506,54);
            Crop(After,580,435,140,180,650,1049,140,180);
            Crop(After,125,255,635,65,867,1075,635,65);
            Crop(After,820,358,90,76,1515,1070,90,76);
            Crop(After,1224,436,88,76,1620,1070,88,76);
            Text("Pen",64,1161,20,Muted);Text("Arrow",650,1240,20,Muted);
            Text("Rectangle",867,1161,20,Muted);Text("Numbers",1515,1161,20,Muted);
            Text("Pressure follows the pen.",64,1322,28,Ink,true);
            Text("Light, heavy, then increasing pressure.",64,1376,20,Muted);
            Text("Synthetic Windows pen input.",64,1410,19,Muted);
            Crop(".tmp/stroke-eval/results/release-020-shapes-1/pressure.png",80,520,980,200,650,1300,980,200);
            Text("Actual 0.2.0 renderings. Themes affect controls; coral annotations stay consistent. Real stylus hardware remains unqualified.",64,1535,18,Muted);
        });
    }
    static Bitmap Chart(double[] before,double[] after) {
        using(var chart=new Chart()) {
            chart.Width=1100;chart.Height=445;chart.BackColor=C(Paper);
            chart.AntiAliasing=AntiAliasingStyles.All;
            var area=new ChartArea("timings");chart.ChartAreas.Add(area);
            area.BackColor=C(Paper);area.AxisX.MajorGrid.Enabled=false;
            area.AxisY.MajorGrid.LineColor=C("#DDE1E9");area.AxisY.LineColor=Color.Transparent;
            area.AxisX.LineColor=C("#BCC3CF");area.AxisX.MajorTickMark.Enabled=false;
            area.AxisY.MajorTickMark.Enabled=false;
            area.AxisY.Minimum=0;area.AxisY.Maximum=150;area.AxisY.Interval=50;
            area.AxisX.Title="Completed long strokes";area.AxisY.Title="p95 paint time (ms)";
            area.AxisX.TitleFont=new Font("Segoe UI",13);area.AxisY.TitleFont=new Font("Segoe UI",13);
            area.AxisX.LabelStyle.Font=new Font("Segoe UI",12);area.AxisY.LabelStyle.Font=new Font("Segoe UI",11);
            area.AxisX.Interval=1;
            foreach(var name in new[]{"0.1.2","0.2.0"}){
                var series=new Series(name);series.ChartType=SeriesChartType.Column;
                series.Color=C(name=="0.1.2"?"#929BAB":Accent);
                series.Font=new Font("Segoe UI",11,FontStyle.Bold);series.IsValueShownAsLabel=true;
                series.LabelFormat="0.0";series["PointWidth"]="0.7";
                double[] values=name=="0.1.2"?before:after;
                for(int i=0;i<4;i++)series.Points.AddXY(new[]{"0","25","100","250"}[i],values[i]);
                chart.Series.Add(series);
            }
            var legend=new Legend();legend.Docking=Docking.Top;legend.Font=new Font("Segoe UI",12);legend.BackColor=C(Paper);chart.Legends.Add(legend);
            var bitmap=new Bitmap(chart.Width,chart.Height);chart.DrawToBitmap(bitmap,new Rectangle(0,0,bitmap.Width,bitmap.Height));return bitmap;
        }
    }
    static void Stress(double[] before,double[] after) {
        Save("03-stress-test.png",1640,()=>{
            Header("More ink. The same drawing pace.","A 240 Hz curve replayed over 250 completed strokes. Actual test captures below.","03 / 03");
            Text("0.1.2  /  baseline",64,224,24,Muted,true);
            Text("0.2.0  /  Natural",916,224,24,Accent,true);
            Crop(".tmp/stroke-eval/results/release-020-baseline-1/curve-250.png",0,0,1400,850,64,272,820,498);
            Crop(".tmp/stroke-eval/results/release-020-natural-1/curve-250.png",0,0,1400,850,916,272,820,498);
            Text("Fast replay loses curve detail at this density.",64,788,21,Muted);
            Text("Captured curve stays within the coverage check.",916,788,21,Muted);
            Fill("#DDE1E9",64,844,1672,1);
            using(var chart=Chart(before,after))g.DrawImage(chart,48,880,1100,445);
            Text(after[3].ToString("0.0",CultureInfo.InvariantCulture)+" ms",1220,934,58,Accent,true);
            Text("p95 paint at 250 strokes",1224,1013,24,Ink,true);
            Text("Baseline: "+before[3].ToString("0.0",CultureInfo.InvariantCulture)+" ms",1224,1061,23,Muted);
            Text("5 alternating runs per build",1224,1130,22,Muted);
            Text("120 observations per density",1224,1168,22,Muted);
            Text("2 seconds per curve at 240 Hz",1224,1206,22,Muted);
            Text("0.2.0: 0 of 9,200 checked reference positions more than 3 px from ink across all four densities.",64,1360,23,Ink,true);
            Text("60-second sustained replay  /  240 Hz  /  25 historical strokes",64,1415,25,Ink,true);
            Text("Packaged executable: all 14,380 checked reference positions within 3 px of ink; max injection lateness 0.62 ms.",64,1459,20,Muted);
            Text("1400 x 850 physical-pixel canvas; 200% display scaling. Forced-paint wall time, not input-to-display latency.",64,1532,19,Muted);
            Text("Single-machine measurements. Coverage is an ink-mask diagnostic, not proof of exact sample retention or unlimited capacity.",64,1571,18,Muted);
        });
    }
    public static void Run(string project,double[] before,double[] after) {
        root=project;BeforeAfter();Components();Stress(before,after);
    }
}
