interface ProgressProps {
    title: string;
    currentStep: number;
    totalSteps: number;
    currentStepName: string;
    percentage?: number;
    className?: string;
}

function Progress({
    title,
    currentStep,
    totalSteps,
    currentStepName,
    percentage,
    className = ''
}: ProgressProps) {
    const progressPercentage = percentage ?? Math.floor((currentStep / totalSteps) * 100);

    return (
        <div className={`progress-container ${className}`}>
            <div className="progress-header">
                <h4 className="progress-title">{title}</h4>
                <span className="progress-percentage">{progressPercentage}%</span>
            </div>

            <div className="progress-bar-container">
                <div
                    className="progress-bar"
                    style={{ width: `${progressPercentage}%` }}
                />
            </div>

            <div className="progress-details">
                <span className="progress-step">
                    Step {currentStep} of {totalSteps}
                </span>
                <span className="progress-current">
                    {currentStepName}
                </span>
            </div>
        </div>
    );
}

export default Progress;