#!/usr/bin/env bash


plog () {
   LOG_TS=`eval date "+%F-%T"`
   echo "[$LOG_TS]: $1"
}

usage() { printf "Usage: $0 \n\t
   -a async\n\t
   -S std\n\t
   -s smol\n\t
   -t tokio\n\t
   -P ping
   -h help\n" 1>&2; exit 1; }

# trap ctrl-c and call ctrl_c()
trap ctrl_c INT

function ctrl_c() {
    cleanup
    exit
}

function std_cleanup() {
   killall std-ping-tcp > /dev/null 2>&1
   killall std-pong-tcp > /dev/null 2>&1
   killall std-ping-udp > /dev/null 2>&1
   killall std-pong-udp > /dev/null 2>&1
}

function async_std_cleanup() {
   killall async-ping-tcp > /dev/null 2>&1
   killall async-pong-tcp > /dev/null 2>&1
   killall async-ping-udp > /dev/null 2>&1
   killall async-pong-udp > /dev/null 2>&1
}

function tokio_cleanup() {
   killall tokio-ping-tcp > /dev/null 2>&1
   killall tokio-pong-tcp > /dev/null 2>&1
   killall tokio-ping-udp > /dev/null 2>&1
   killall tokio-pong-udp > /dev/null 2>&1
}

function smol_cleanup() {
   killall smol-ping-tcp > /dev/null 2>&1
   killall smol-pong-tcp > /dev/null 2>&1
   killall smol-ping-udp > /dev/null 2>&1
   killall smol-pong-udp > /dev/null 2>&1
}

# kills all the processes
function cleanup() {
   std_cleanup
   async_std_cleanup
   tokio_cleanup
   smol_cleanup
}



if [[ ! $@ =~ ^\-.+ ]]
then
  usage;
fi



TS=$(date +%Y%m%d.%H%M%S)

N_CPU=$(nproc)

INITIAL_MSGS=1


OUT_DIR="${OUT_DIR:-latency-logs}"
INTERVALS=(1 0.1 0.001 0.0001 0.00001 0.000001 0.0000001 0)
#INTERVALS=(1 0.1 0.001 0.0001)
TASKS=(0 10 100 1000)

DURATION=${DURATION:-60}
SIZE=${SIZE:-64}
CPUS="${CPUS:-0,1}"
TASKS="${TASKS:-0}"
NICE="${NICE:--10}"


TCP_PING_REMOTE="127.0.0.1:9009"
TCP_PING_LOCAL="127.0.0.1:9009"
UDP_PING_REMOTE="127.0.0.1:9009"
UDP_PING_LOCAL="127.0.0.1:9999"
ICMP_REMOTE="127.0.0.1"



while getopts "asSthP" arg; do
   case ${arg} in
   h)
      usage
      ;;
   a)
      # async std

      plog "[ START ] async_std latency test"

      #tcp
      for i in "${INTERVALS[@]}"
      do
         for t in "${TASKS[@]}"
         do

            plog "[ START ] async_std tcp with interval $i and tasks $t"
            NICE=$NICE CPUS=0 SIZE=$SIZE LOCAL=$TCP_PING_LOCAL ./run-single-process.sh -oa &
            PONG_PID=$!
            sleep 2
            DURATION=$DURATION CPUS=1 NICE=$NICE REMOTE=$TCP_PING_REMOTE SIZE=$SIZE TASKS=$t INTERVAL=$i  ./run-single-process.sh -ia

            kill -9 $PONG_PID
            async_std_cleanup
            plog "[ DONE ] async_std tcp with interval $i and tasks $t"
            sleep 2
         done
         cleanup
      done

      #udp
      for i in "${INTERVALS[@]}"
      do
         for t in "${TASKS[@]}"
         do

            plog "[ START ] async_std udp with interval $i and tasks $t"
            NICE=$NICE CPUS=0 SIZE=$SIZE LOCAL=$UDP_PING_REMOTE ./run-single-process.sh -Oa &
            PONG_PID=$!
            sleep 2
            DURATION=$DURATION CPUS=1 NICE=$NICE LOCAL=$UDP_PING_LOCAL REMOTE=$UDP_PING_REMOTE SIZE=$SIZE TASKS=$t INTERVAL=$i  ./run-single-process.sh -Ia

            kill -9 $PONG_PID
            async_std_cleanup
            plog "[ DONE ] async_std udp with interval $i and tasks $t"
            sleep 2
         done
         cleanup
      done

      plog "[ END ] async_std latency test"
      ;;
   S)
      # std

     plog "[ START ] std latency test"

      #tcp
      for i in "${INTERVALS[@]}"
      do
         for t in "${TASKS[@]}"
         do

            plog "[ START ] std tcp with interval $i and tasks $t"
            NICE=$NICE CPUS=0 SIZE=$SIZE LOCAL=$TCP_PING_LOCAL ./run-single-process.sh -oS &
            PONG_PID=$!
            sleep 2
            DURATION=$DURATION CPUS=1 NICE=$NICE REMOTE=$TCP_PING_REMOTE SIZE=$SIZE TASKS=$t INTERVAL=$i  ./run-single-process.sh -iS

            kill -9 $PONG_PID
            std_cleanup
            plog "[ DONE ] std tcp with interval $i and tasks $t"
            sleep 2
         done
         cleanup
      done

      #udp
      for i in "${INTERVALS[@]}"
      do
         for t in "${TASKS[@]}"
         do

            plog "[ START ] std udp with interval $i and tasks $t"
            NICE=$NICE CPUS=0 SIZE=$SIZE LOCAL=$TCP_PING_LOCAL ./run-single-process.sh -OS &
            PONG_PID=$!
            sleep 2
            DURATION=$DURATION CPUS=1 NICE=$NICE LOCAL=$UDP_PING_LOCAL REMOTE=$UDP_PING_REMOTE SIZE=$SIZE TASKS=$t INTERVAL=$i  ./run-single-process.sh -IS

            kill -9 $PONG_PID
            std_cleanup
            plog "[ DONE ] std udp with interval $i and tasks $t"
            sleep 2
         done
         cleanup
      done


      plog "[ END ] std latency test"
      ;;
   s)
      # smol


      plog "[ START ] smol latency test"

      #tcp
      for i in "${INTERVALS[@]}"
      do
         for t in "${TASKS[@]}"
         do

            plog "[ START ] smol tcp with interval $i and tasks $t"
            NICE=$NICE CPUS=0 SIZE=$SIZE LOCAL=$TCP_PING_LOCAL ./run-single-process.sh -os &
            PONG_PID=$!
            sleep 2
            DURATION=$DURATION CPUS=1 NICE=$NICE REMOTE=$TCP_PING_REMOTE SIZE=$SIZE TASKS=$t INTERVAL=$i  ./run-single-process.sh -is

            kill -9 $PONG_PID
            smol_cleanup
            plog "[ DONE ] std tcp with interval $i and tasks $t"
            sleep 2
         done
         cleanup
      done

      #udp
      for i in "${INTERVALS[@]}"
      do
         for t in "${TASKS[@]}"
         do

            plog "[ START ] smol udp with interval $i and tasks $t"
            NICE=$NICE CPUS=0 SIZE=$SIZE LOCAL=$TCP_PING_LOCAL ./run-single-process.sh -Os &
            PONG_PID=$!
            sleep 2
            DURATION=$DURATION CPUS=1 NICE=$NICE LOCAL=$UDP_PING_LOCAL REMOTE=$UDP_PING_REMOTE SIZE=$SIZE TASKS=$t INTERVAL=$i  ./run-single-process.sh -Is

            kill -9 $PONG_PID
            smol_cleanup
            plog "[ DONE ] smol udp with interval $i and tasks $t"
            sleep 2
         done
         cleanup
      done

      plog "[ END ] smol latency test"
      ;;
   t)
      # tokio

      plog "[ START ] tokio latency test"

      #tcp
      for i in "${INTERVALS[@]}"
      do
         for t in "${TASKS[@]}"
         do

            plog "[ START ] tokio tcp with interval $i and tasks $t"
            NICE=$NICE CPUS=0 SIZE=$SIZE LOCAL=$TCP_PING_LOCAL ./run-single-process.sh -ot &
            PONG_PID=$!
            sleep 2
            DURATION=$DURATION CPUS=1 NICE=$NICE REMOTE=$TCP_PING_REMOTE SIZE=$SIZE TASKS=$t INTERVAL=$i  ./run-single-process.sh -it

            kill -9 $PONG_PID
            tokio_cleanup
            plog "[ DONE ] tokio tcp with interval $i and tasks $t"
            sleep 2
         done
         cleanup
      done

      #udp
      for i in "${INTERVALS[@]}"
      do
         for t in "${TASKS[@]}"
         do

            plog "[ START ] tokio udp with interval $i and tasks $t"
            NICE=$NICE CPUS=0 SIZE=$SIZE LOCAL=$UDP_PING_REMOTE ./run-single-process.sh -Ot &
            PONG_PID=$!
            sleep 2
            DURATION=$DURATION CPUS=1 NICE=$NICE LOCAL=$UDP_PING_LOCAL REMOTE=$UDP_PING_REMOTE SIZE=$SIZE TASKS=$t INTERVAL=$i  ./run-single-process.sh -It

            kill -9 $PONG_PID
            tokio_cleanup
            plog "[ DONE ] tokio udp with interval $i and tasks $t"
            sleep 2
         done
         cleanup
      done

      plog "[ END ] tokio latency test"
      ;;
   P)
      plog "[ START ] ping latency test"

      #icmp ping
      for i in "${INTERVALS[@]}"
      do

         plog "[ START ] ping with interval $i and tasks $t"
         DURATION=$DURATION CPUS=1 NICE=$NICE ICMP_REMOTE=$ICMP_REMOTE SIZE=$SIZE INTERVAL=$i  ./run-single-process.sh -P

         plog "[ DONE ] ping with interval $i and tasks $t"
         sleep 2

         cleanup
      done
      plog "[ END ] ping latency test"
      ;;
   *)
      usage
      ;;
   esac
done

cleanup
plog "[ DONE ] Bye!"

