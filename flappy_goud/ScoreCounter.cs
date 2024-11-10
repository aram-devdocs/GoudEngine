using CsBindgen;

public class ScoreCounter
{
    public int Score { get; private set; }

    public ScoreCounter()
    {
        Score = 0;
    }

    public void IncrementScore()
    {
        Score++;
    }

    public void ResetScore()
    {
        Score = 0;
    }


}