#!/usr/bin/env bash


plog () {
   LOG_TS=`eval date "+%F-%T"`
   echo "[$LOG_TS]: $1"
}

usage() { printf "Usage: $0 \n\t
   -i ping tcp\n\t
   -o pong tcp\n\t
   -I ping udp\n\t
   -O pong tcp\n\t
   -S std\n\t
   -a async_std\n\t
   -t tokio\n\t
   -s smol\n\t
   -P ICMP ping\n\t
   -h help\n" 1>&2; exit 1; }



if [[ ! $@ =~ ^\-.+ ]]
then
  usage;
fi



TS=$(date +%Y%m%d.%H%M%S)

N_CPU=$(nproc)

CHAIN_LENGTH=1
BIN_DIR="./target/release"


WD=$(pwd)

ASYNC_PING_TCP="async-ping-tcp"
ASYNC_PONG_TCP="async-pong-tcp"
ASYNC_PING_UDP="async-ping-udp"
ASYNC_PONG_UDP="async-pong-udp"

SMOL_PING_TCP="smol-ping-tcp"
SMOL_PONG_TCP="smol-pong-tcp"
SMOL_PING_UDP="smol-ping-udp"
SMOL_PONG_UDP="smol-pong-udp"

STD_PING_TCP="std-ping-tcp"
STD_PONG_TCP="std-pong-tcp"
STD_PING_UDP="std-ping-udp"
STD_PONG_UDP="std-pong-udp"

TOKIO_PING_TCP="tokio-ping-tcp"
TOKIO_PONG_TCP="tokio-pong-tcp"
TOKIO_PING_UDP="tokio-ping-udp"
TOKIO_PONG_UDP="tokio-pong-udp"



OUT_DIR="${OUT_DIR:-latency-logs}"
INTERVAL=${INTERVAL:-1}
DURATION=${DURATION:-60}
SIZE=${SIZE:-64}
CPUS="${CPUS:-0,1}"
TASKS="${TASKS:-0}"
NICE="${NICE:--10}"

LOCAL="${LOCAL:-127.0.0.1:9009}"
REMOTE="${REMOTE:-127.0.0.1:9999}"

ICMP_REMOTE="${ICMP_REMOTE:-127.0.0.1}"


mkdir -p $OUT_DIR

# Run source by default:
# - 1 = Ping TCP
# - 2 = Pong TCP
# - 3 = Ping UDP
# - 4 = Pong UDP
TORUN=1


plog "[ INIT ] Duration will be $DURATION seconds"
plog "[ INIT ] Sending a message each $INTERVAL"
plog "[ INIT ] Message size $SIZE bytes"
while getopts "iIoOsSathP" arg; do
   case ${arg} in
   h)
      usage
      ;;
   i)
      # Start ping tcp

      plog "[ INIT ] Running the ping tcp"
      TORUN=1
      ;;
   o)
      # Start pong tcp

      plog "[ INIT ] Running the pong tcp"
      TORUN=2
      ;;
   I)
      # Start ping udp

      plog "[ INIT ] Running the ping udp"
      TORUN=3
      ;;
   O)
      # Start pong udp

      plog "[ INIT ] Running the pong udp"
      TORUN=4
      ;;
   S)
      # std
      case ${TORUN} in
      1)
         LOG_FILE="$OUT_DIR/std-ping-tcp-$TS-$TASKS-$SIZE-$INTERVAL.csv"
         echo "framework,transport,test,count,rate,payload,tasks,value,unit" > $LOG_FILE
         plog "[ RUN ] Running std ping tcp"
         timeout $DURATION nice $NICE taskset -c $CPUS $BIN_DIR/$STD_PING_TCP $REMOTE $SIZE $INTERVAL -w -c -s $TASKS >> $LOG_FILE 2> /dev/null
         plog "[ DONE ] Running std ping tcp"
         ;;
      2)
         plog "[ RUN ] Running std pong tcp"
         nice $NICE taskset -c $CPUS $BIN_DIR/$STD_PONG_TCP $LOCAL $SIZE > /dev/null 2>&1
         plog "[ DONE ] Running std pong tcp"
         ;;
      3)
         LOG_FILE="$OUT_DIR/std-ping-udp-$TS-$TASKS-$SIZE-$INTERVAL.csv"
         echo "framework,transport,test,count,rate,payload,tasks,value,unit" > $LOG_FILE
         plog "[ RUN ] Running std ping udp"
         timeout $DURATION nice $NICE taskset -c $CPUS $BIN_DIR/$STD_PING_UDP $LOCAL $REMOTE $SIZE $INTERVAL -w -c -s $TASKS >> $LOG_FILE 2> /dev/null
         plog "[ DONE ] Running std ping udp"
         ;;
      4)
         plog "[ RUN ] Running std pong udp"
         nice $NICE taskset -c $CPUS $BIN_DIR/$STD_PONG_UDP $LOCAL $SIZE > /dev/null 2>&1
         plog "[ DONE ] Running std pong udp"
         ;;
      *)
         usage
         ;;
      esac
      ;;

   a)
      # async-std
      case ${TORUN} in
      1)
         LOG_FILE="$OUT_DIR/async_std-ping-tcp-$TS-$TASKS-$SIZE-$INTERVAL.csv"
         echo "framework,transport,test,count,rate,payload,tasks,value,unit" > $LOG_FILE
         plog "[ RUN ] Running async_std ping tcp"
         timeout $DURATION nice $NICE taskset -c $CPUS $BIN_DIR/$ASYNC_PING_TCP $REMOTE $SIZE $INTERVAL -w -c -s $TASKS >> $LOG_FILE 2> /dev/null
         plog "[ DONE ] Running async_std ping tcp"
         ;;
      2)
         plog "[ RUN ] Running async_std pong tcp"
         nice $NICE taskset -c $CPUS $BIN_DIR/$ASYNC_PONG_TCP $LOCAL $SIZE > /dev/null 2>&1
         plog "[ DONE ] Running async_std pong tcp"
         ;;
      3)
         LOG_FILE="$OUT_DIR/async_std-ping-udp-$TS-$TASKS-$SIZE-$INTERVAL.csv"
         echo "framework,transport,test,count,rate,payload,tasks,value,unit" > $LOG_FILE
         plog "[ RUN ] Running async_std ping udp"
         timeout $DURATION nice $NICE taskset -c $CPUS $BIN_DIR/$ASYNC_PING_UDP $LOCAL $REMOTE $SIZE $INTERVAL -w -c -s $TASKS >> $LOG_FILE 2> /dev/null
         plog "[ DONE ] Running async_std ping udp"
         ;;
      4)
         plog "[ RUN ] Running async_std pong udp"
         nice $NICE taskset -c $CPUS $BIN_DIR/$ASYNC_PONG_UDP $LOCAL $SIZE > /dev/null 2>&1
         plog "[ DONE ] Running async_std pong udp"
         ;;
      *)
         usage
         ;;
      esac
      ;;

   t)
      # tokio
      case ${TORUN} in
      1)
         LOG_FILE="$OUT_DIR/tokio-ping-tcp-$TS-$TASKS-$SIZE-$INTERVAL.csv"
         echo "framework,transport,test,count,rate,payload,tasks,value,unit" > $LOG_FILE
         plog "[ RUN ] Running tokio ping tcp"
         timeout $DURATION nice $NICE taskset -c $CPUS $BIN_DIR/$TOKIO_PING_TCP $REMOTE $SIZE $INTERVAL -w -c -s $TASKS >> $LOG_FILE 2> /dev/null
         plog "[ DONE ] Running tokio ping tcp"
         ;;
      2)
         plog "[ RUN ] Running tokio pong tcp"
         nice $NICE taskset -c $CPUS $BIN_DIR/$TOKIO_PONG_TCP $LOCAL $SIZE > /dev/null 2>&1
         plog "[ DONE ] Running tokio pong tcp"
         ;;
      3)
         LOG_FILE="$OUT_DIR/tokio-ping-udp-$TS-$TASKS-$SIZE-$INTERVAL.csv"
         echo "framework,transport,test,count,rate,payload,tasks,value,unit" > $LOG_FILE
         plog "[ RUN ] Running tokio ping udp"
         timeout $DURATION nice $NICE taskset -c $CPUS $BIN_DIR/$TOKIO_PING_UDP $LOCAL $REMOTE $SIZE $INTERVAL -w -c -s $TASKS >> $LOG_FILE 2> /dev/null
         plog "[ DONE ] Running tokio ping udp"
         ;;
      4)
         plog "[ RUN ] Running tokio pong udp"
         nice $NICE taskset -c $CPUS $BIN_DIR/$TOKIO_PONG_UDP $LOCAL $SIZE > /dev/null 2>&1
         plog "[ DONE ] Running tokio pong udp"
         ;;
      *)
         usage
         ;;
      esac
      ;;
   s)
      # smol
      case ${TORUN} in
      1)
         LOG_FILE="$OUT_DIR/smol-ping-tcp-$TS-$TASKS-$SIZE-$INTERVAL.csv"
         echo "framework,transport,test,count,rate,payload,tasks,value,unit" > $LOG_FILE
         plog "[ RUN ] Running smol ping tcp"
         timeout $DURATION nice $NICE taskset -c $CPUS $BIN_DIR/$SMOL_PING_TCP $REMOTE $SIZE $INTERVAL -w -c -s $TASKS >> $LOG_FILE 2> /dev/null
         plog "[ DONE ] Running smol ping tcp"
         ;;
      2)
         plog "[ RUN ] Running smol pong tcp"
         nice $NICE taskset -c $CPUS $BIN_DIR/$SMOL_PONG_TCP $LOCAL $SIZE > /dev/null 2>&1
         plog "[ DONE ] Running smol pong tcp"
         ;;
      3)
         LOG_FILE="$OUT_DIR/smol-ping-udp-$TS-$TASKS-$SIZE-$INTERVAL.csv"
         echo "framework,transport,test,count,rate,payload,tasks,value,unit" > $LOG_FILE
         plog "[ RUN ] Running smol ping udp"
         timeout $DURATION nice $NICE taskset -c $CPUS $BIN_DIR/$SMOL_PING_UDP $LOCAL $REMOTE $SIZE $INTERVAL -w -c -s $TASKS >> $LOG_FILE 2> /dev/null
         plog "[ DONE ] Running smol ping udp"
         ;;
      4)
         plog "[ RUN ] Running smol pong udp"
         nice $NICE taskset -c $CPUS $BIN_DIR/$SMOL_PONG_UDP $LOCAL $SIZE > /dev/null 2>&1
         plog "[ DONE ] Running smol pong udp"
         ;;
      *)
         usage
         ;;
      esac
      ;;
   P)
      plog "[ RUN ] ICMP ping"
      LOG_FILE="$OUT_DIR/icmp-ping-$TS-$TASKS-$SIZE-$INTERVAL.csv"
      echo "framework,transport,test,count,rate,payload,tasks,value,unit" > $LOG_FILE
      sudo timeout $DURATION nice $NICE taskset -c $CPUS ping $ICMP_REMOTE -i $INTERVAL | awk -v intv=$INTERVAL 'BEGIN {FS="[=]|[ ]"} NR>=2 {printf("ping,icmp,rtt,%d,%f,64,0,%s,%s\n",$6,intv,$10,$11)}' >> $LOG_FILE 2> /dev/null
      plog "[ DONE ] ICMP ping"
      ;;
   *)
      usage
      ;;
   esac
done

plog "Bye!"

