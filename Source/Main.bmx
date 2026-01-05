SuperStrict

Framework brl.standardio
Import brl.linkedlist
Import brl.textstream
Import brl.filesystem

Type River
    Field name:String
    Field country:String
    Field emission:Float
    Field probability:Float
    Field popImpact:Float
    Field cost:Float
    Field karmaScore:Float
End Type

Function CalculateKarma:Float(emission:Float, prob:Float, pop:Float, cost:Float)
    Local maxCost:Float = 1000000.0
    Local costEfficiency:Float = 1.0 - (cost / maxCost)
    If costEfficiency < 0 Then costEfficiency = 0
    Return 0.4 * emission / 40000.0 + 0.3 * prob + 0.2 * pop / 10.0 + 0.1 * costEfficiency
End Function

Local rivers:TList = New TList
Local file:TTextStream = ReadFile("Data/GlobalRiverPlastic_Meijer2021.csv")
If Not file Then RuntimeError "CSV file not found"

Local header:String = file.ReadLine() ' skip header

While Not file.Eof()
    Local line:String = file.ReadLine()
    Local fields:String[] = line.Split(",")
    If fields.length < 9 Then Continue
    
    Local r:River = New River
    r.name = fields[1]
    r.country = fields[2]
    r.emission = Float(fields[5])
    r.probability = Float(fields[6])
    r.popImpact = Float(fields[7])
    r.cost = Float(fields[8])
    r.karmaScore = CalculateKarma(r.emission, r.probability, r.popImpact, r.cost)
    rivers.AddLast(r)
Wend
file.Close()

' Sort descending by karma
rivers.Sort(True, CompareRivers)

Print "Top 10 Priority Rivers for Cleanup Deployment:"
Local count:Int = 0
For Local r:River = EachIn rivers
    Print $"{r.name} ({r.country}): KarmaScore = {r.karmaScore:.3f} | Emission = {r.emission:.0f} MT/year"
    count :+ 1
    If count = 10 Then Exit
Next

Function CompareRivers:Int(a:Object, b:Object)
    Local ra:River = River(a)
    Local rb:River = River(b)
    Return FloatCompare(rb.karmaScore, ra.karmaScore)
End Function

Function FloatCompare:Int(a:Float, b:Float)
    If a > b Return 1
    If a < b Return -1
    Return 0
End Function
