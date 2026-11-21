package org.codehaus.plexus.build.progress;

import javax.inject.Inject;
import javax.inject.Named;

import org.apache.maven.execution.scope.MojoExecutionScoped;
import org.apache.maven.plugin.MojoExecution;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

/**
 * The default implementation simply log to debug and check for thread
 * interruption
 */
@Named("default")
@MojoExecutionScoped
public class DefaultProgress implements Progress {

    private final Logger logger = LoggerFactory.getLogger(DefaultProgress.class);
    private MojoExecution execution;
    private int work;

    @Inject
    public DefaultProgress(MojoExecution execution) {
        this.execution = execution;
    }

    @Override
    public void startTask(String task, int work) {
        setRemaining(work);
        logger.debug(execution.getExecutionId() + ": " + task);
    }

    @Override
    public void worked(int work) {
        if (work == 0) {
            return;
        }
        if (work < 0) {
            logger.warn(execution.getExecutionId() + " reported negative amount of work!");
        }
        if (this.work < 0) {
            return;
        }
        if (work > this.work) {
            this.work = -1;
            logger.warn(execution.getExecutionId() + " reported more work than expected!");
        } else {
            this.work -= work;
        }
    }

    @Override
    public void setRemaining(int work) {
        this.work = work <= 0 ? -1 : work;
    }

    @Override
    public boolean isCancelRequested() {
        return Thread.currentThread().isInterrupted();
    }
}
