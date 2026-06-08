import sn0rk/utils;

isolated function mapFineResult(AnswerFineInfo answer, string env) returns ResultFine|error {
    ResultFine resultFine = check prepareFineResult(answer: answer);

    if resultFine.typeFine == 2 || resultFine.typeFine == 3 || resultFine.typeFine == 4 {
    } else {
        resultFine = check handlePenaltyIncrease(resultFine: resultFine);
    }

    if answer.penalty?.DPAYTERM == () && VerifyPay(docId: answer.docId, env: env) == "True" {
        resultFine.status = "ОЧІКУЄТЬСЯ ПІДТВЕРДЖЕННЯ ОПЛАТИ З ДЕРЖАВНОЇ КАЗНАЧЕЙСЬКОЇ СЛУЖБИ";
        resultFine.paidinfo = ();
    }

    if answer.processStatus?.statusId == "7" {
        resultFine.status = answer.processStatus?.status.toString().toUpperAscii();
    }

    return resultFine;
}

isolated function mapFinePedestrianResult(AnswerFineInfo answer, string env) returns ResultFine|error {
    ResultFine resultFine = check prepareFineResult(answer: answer);

    resultFine = check handlePenaltyIncrease(resultFine: resultFine);

    if answer.penalty?.DPAYTERM == () && VerifyPay(docId: answer.docId, env: env) == "True" {
        resultFine.status = "ОЧІКУЄТЬСЯ ПІДТВЕРДЖЕННЯ ОПЛАТИ З ДЕРЖАВНОЇ КАЗНАЧЕЙСЬКОЇ СЛУЖБИ";
        resultFine.paidinfo = ();
    }
}
